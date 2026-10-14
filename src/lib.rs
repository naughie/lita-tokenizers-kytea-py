#[pyo3::pymodule]
mod lita_tokenizers_kytea_py {
    use pyo3::exceptions as pyerr;
    use pyo3::prelude::*;

    use kytea_sys::{CorpusFormat, DebugLevel, ModelFormat};

    use std::ffi::{CStr, CString};

    #[pyclass(unsendable)]
    struct KyTea {
        // `ffi` must be dropped first
        ffi: kytea_sys::KyTea,
        _cptrs: OwnedCPtrs,
    }

    struct OwnedCPtr(Option<Box<CStr>>);

    impl OwnedCPtr {
        fn new(c: Option<char>) -> Self {
            let Some(c) = c else {
                return Self(None);
            };
            if c == '\0' {
                return Self(None);
            }

            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);

            let encoded_len = encoded.len();

            let mut inner = Vec::with_capacity(encoded_len + 1);
            inner[..encoded_len].copy_from_slice(encoded.as_bytes());
            inner[encoded_len] = 0;

            // SAFETY: `c != '\0'` guarantees that `encoded` contains no 0u8
            let inner = unsafe { CString::from_vec_with_nul_unchecked(inner) };
            Self(Some(inner.into_boxed_c_str()))
        }

        fn as_cstr(&self) -> Option<&CStr> {
            self.0.as_deref()
        }
    }

    struct OwnedCPtrs {
        word_bound: OwnedCPtr,
        tag_bound: OwnedCPtr,
        elem_bound: OwnedCPtr,
        unk_bound: OwnedCPtr,
        no_bound: OwnedCPtr,
        has_bound: OwnedCPtr,
        skip_bound: OwnedCPtr,
        escape: OwnedCPtr,
    }

    #[pymethods]
    impl KyTea {
        #[new]
        #[pyo3(signature = (
            debug="silent", input_format="raw",
            word_bound=None, tag_bound=None, elem_bound=None,
            unk_bound=None, no_bound=None, has_bound=None, skip_bound=None,
            escape=None,
        ))]
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            debug: Option<&str>,
            input_format: Option<&str>,
            word_bound: Option<char>,
            tag_bound: Option<char>,
            elem_bound: Option<char>,
            unk_bound: Option<char>,
            no_bound: Option<char>,
            has_bound: Option<char>,
            skip_bound: Option<char>,
            escape: Option<char>,
        ) -> Self {
            let debug = match debug {
                Some("silent") => DebugLevel::Silent,
                Some("simple") => DebugLevel::Simple,
                Some("detailed") => DebugLevel::Detailed,
                Some("full") => DebugLevel::Full,
                _ => DebugLevel::Silent,
            };
            let input_format = match input_format {
                Some("raw") => CorpusFormat::Raw,
                Some("full") => CorpusFormat::Full,
                Some("part") => CorpusFormat::Part,
                Some("prob") => CorpusFormat::Prob,
                Some("tok") => CorpusFormat::Tok,
                Some("eda") => CorpusFormat::Eda,
                Some("tags") => CorpusFormat::Tags,
                _ => CorpusFormat::Raw,
            };

            let ptrs = OwnedCPtrs {
                word_bound: OwnedCPtr::new(word_bound),
                tag_bound: OwnedCPtr::new(tag_bound),
                elem_bound: OwnedCPtr::new(elem_bound),
                unk_bound: OwnedCPtr::new(unk_bound),
                no_bound: OwnedCPtr::new(no_bound),
                has_bound: OwnedCPtr::new(has_bound),
                skip_bound: OwnedCPtr::new(skip_bound),
                escape: OwnedCPtr::new(escape),
            };

            let mut ffi = kytea_sys::KyTea::new();
            let mut config = ffi.config();
            config
                .set_debug(debug)
                .set_training(false)
                .set_input_format(input_format);

            // SAFETY: the signature of `KyTea` guarantees that `ptrs` outlives the FFI model.
            unsafe {
                if let Some(ptr) = ptrs.word_bound.as_cstr() {
                    config.set_word_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.tag_bound.as_cstr() {
                    config.set_tag_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.elem_bound.as_cstr() {
                    config.set_elem_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.unk_bound.as_cstr() {
                    config.set_unk_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.no_bound.as_cstr() {
                    config.set_no_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.has_bound.as_cstr() {
                    config.set_has_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.skip_bound.as_cstr() {
                    config.set_skip_bound_unchecked(ptr);
                }
                if let Some(ptr) = ptrs.escape.as_cstr() {
                    config.set_escape_unchecked(ptr);
                }
            }

            Self { ffi, _cptrs: ptrs }
        }

        pub fn read_model(&mut self, path: &str) -> PyResult<()> {
            let Ok(model_c) = CString::new(path) else {
                return Err(PyErr::new::<pyerr::PyValueError, _>(
                    "could not convert the model path to CString",
                ));
            };

            self.ffi.read_model(&model_c, ModelFormat::Unknown)?;

            Ok(())
        }

        pub fn read_model_from_bytes(&mut self, bytes: &[u8], format: &str) -> PyResult<()> {
            use kytea_sys::IspanStream;

            let format = match format {
                "t" | "txt" | "text" => ModelFormat::Text,
                "b" | "bin" | "binary" => ModelFormat::Binary,
                _ => {
                    return Err(PyErr::new::<pyerr::PyValueError, _>(
                        "model format is eiher \"t\" (text) or \"b\" (binary)",
                    ));
                }
            };

            let mut stream = IspanStream::new(bytes);
            self.ffi.read_model_from_stream(&mut stream, format)?;

            Ok(())
        }

        pub fn tokenize(
            &mut self,
            input: Bound<'_, Stream>,
            output: Bound<'_, Stream>,
        ) -> PyResult<()> {
            let input_as_stream = input.call_method0("bare")?;
            let mut input_resolved = input_as_stream.extract::<ResolvedStream>()?;
            let mut input_stream: &mut dyn kytea_sys::Stream = match &mut input_resolved {
                ResolvedStream::StringStream(s) => {
                    if !s.ffi.as_bytes().ends_with(b"\n") {
                        s.ffi.push("\n");
                    }
                    &mut s.ffi
                }
                ResolvedStream::FileStream(s) => {
                    if !s.input {
                        return Err(PyErr::new::<pyerr::PyValueError, _>(
                            "input file must be opened with the \"r\" mode",
                        ));
                    }
                    &mut s.ffi
                }
            };

            let output_as_stream = output.call_method0("bare")?;
            let mut output_resolved = output_as_stream.extract::<ResolvedStream>()?;
            let mut output_stream: &mut dyn kytea_sys::Stream = match &mut output_resolved {
                ResolvedStream::StringStream(s) => &mut s.ffi,
                ResolvedStream::FileStream(s) => {
                    if s.input {
                        return Err(PyErr::new::<pyerr::PyValueError, _>(
                            "output file must be opened with the \"w\" or \"a\" mode",
                        ));
                    }
                    &mut s.ffi
                }
            };

            let mut ctx = self
                .ffi
                .context(&mut input_stream, &mut output_stream)
                .map_err(|e| {
                    PyErr::new::<pyerr::PyRuntimeError, _>(format!(
                        "tokenization config is wrong: {e}"
                    ))
                })?;
            while ctx.predict()?.is_continue() {}

            Ok(())
        }
    }

    #[pyclass(subclass, unsendable)]
    pub struct Stream;

    #[pyclass(extends=Stream, unsendable)]
    pub struct StringStream {
        ffi: kytea_sys::StringStream,
    }

    #[pyclass(extends=Stream, unsendable)]
    pub struct FileStream {
        ffi: kytea_sys::Fstream,
        input: bool,
    }

    #[derive(FromPyObject)]
    pub enum ResolvedStream<'py> {
        StringStream(PyRefMut<'py, StringStream>),
        FileStream(PyRefMut<'py, FileStream>),
    }

    #[pymethods]
    impl Stream {
        #[new]
        pub fn new() -> Self {
            Stream
        }

        pub fn bare(slf: Bound<'_, Self>) -> Bound<'_, PyAny> {
            slf.into_any()
        }
    }

    #[pymethods]
    impl StringStream {
        #[new]
        #[pyo3(signature = (string=None))]
        pub fn new(string: Option<&str>) -> PyClassInitializer<Self> {
            let mut ffi = kytea_sys::StringStream::new();
            if let Some(s) = string {
                ffi.push(s);
            }
            PyClassInitializer::from(Stream).add_subclass(Self { ffi })
        }

        pub fn as_str(&self) -> PyResult<&str> {
            std::str::from_utf8(self.ffi.as_bytes()).map_err(|e| {
                PyErr::new::<pyerr::PyUnicodeDecodeError, _>(format!(
                    "tokenization output is invalid UTF-8: {e}"
                ))
            })
        }
    }

    #[pymethods]
    impl FileStream {
        #[new]
        pub fn new(path: &str, mode: &str) -> PyResult<PyClassInitializer<Self>> {
            let path = || {
                CString::new(path).map_err(|_| {
                    PyErr::new::<pyerr::PyValueError, _>(
                        "could not convert the file path to CString",
                    )
                })
            };
            let (ffi, input) = match mode {
                "r" => (kytea_sys::Fstream::open(&path()?)?, true),
                "w" => (kytea_sys::Fstream::create(&path()?)?, false),
                "a" => (kytea_sys::Fstream::append(&path()?)?, false),
                _ => {
                    return Err(PyErr::new::<pyerr::PyValueError, _>(
                        r#"file mode must be either "r", "w", or "a""#,
                    ));
                }
            };
            Ok(PyClassInitializer::from(Stream).add_subclass(Self { ffi, input }))
        }

        pub fn flush(&mut self) -> PyResult<()> {
            self.ffi.flush()?;
            Ok(())
        }
    }
}
