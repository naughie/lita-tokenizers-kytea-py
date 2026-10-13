#[pyo3::pymodule]
mod lita_tokenizers_kytea_py {
    use pyo3::exceptions as pyerr;
    use pyo3::prelude::*;

    use kytea_sys::ModelFormat;
    use tokenizers_core::delim;

    use std::ffi::CString;

    #[pyclass(unsendable)]
    struct KyTea(kytea_sys::KyTea);

    #[pymethods]
    impl KyTea {
        #[new]
        pub fn new() -> Self {
            use kytea_sys::{CorpusFormat, DebugLevel};

            let mut ffi = kytea_sys::KyTea::new();
            ffi.config()
                .set_debug(DebugLevel::Silent)
                .set_training(false)
                .set_input_format(CorpusFormat::Raw)
                .set_word_bound(delim::TOKEN_CSTR)
                .set_tag_bound(delim::TAG_CSTR)
                .set_elem_bound(delim::TOKEN_CSTR)
                .set_escape(delim::ESCAPE_CSTR);

            Self(ffi)
        }

        pub fn read_model(&mut self, path: &str) -> PyResult<()> {
            let Ok(model_c) = CString::new(path) else {
                return Err(PyErr::new::<pyerr::PyValueError, _>(
                    "could not convert the model path to CString",
                ));
            };

            self.0.read_model(&model_c, ModelFormat::Unknown)?;

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
            self.0.read_model_from_stream(&mut stream, format)?;

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
                .0
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
