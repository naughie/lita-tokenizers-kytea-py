# lita-tokenizers-kytea-py

KyTea bindings for Python.


## Usage

Install via PyPI:

```
$ pip install lita-tokenizers-kytea-py
```

Then you can use `KyTea` for performing the morphological analysis.

```python
from lita_tokenizers_kytea_py import KyTea, StringStream


def main():
    model = KyTea()
    model.read_model("/path/to/model.bin")

    input = StringStream("すもももももももものうち")
    output = StringStream()

    model.tokenize(input, output)
    print(output.as_str())


if __name__ == "__main__":
    main()
```

Both the input and the output are allowed to be files:

```python
from lita_tokenizers_kytea_py import FileStream, KyTea


def main():
    model = KyTea()
    model.read_model("/path/to/model.bin")

    model = KyTea()
    model.read_model(model_path)

    # it accepts three modes:
    # "r" (read-only), "w" (write-only, truncate), "a" (write-only, append)
    input = FileStream("./input.txt", "r")
    output = FileStream("./output.txt", "w")

    model.tokenize(input, output)
    output.flush()
```

The format of models can be text-based (trained with the `-modtext` option):

```python
from lita_tokenizers_kytea_py import KyTea, StringStream


def main():
    model = KyTea()
    model.read_model("/path/to/model.txt")

    input = StringStream("すもももももももものうち")
    output = StringStream()

    model.tokenize(input, output)
    print(output.as_str())


if __name__ == "__main__":
    main()
```

Also, the model itself can be loaded before calling `read_model()`:

```python
from lita_tokenizers_kytea_py import FileStream, KyTea


def main():
    model = KyTea()
    with open("/path/to/model.bin", "rb") as f:
        # "b": binary model
        # "t": text model
        model.read_model_from_bytes(f.read(), "b")

    input = StringStream("すもももももももものうち")
    output = StringStream()

    model.tokenize(input, output)
    print(output.as_str())
```

### Configs

You can pass the KyTea's config parameters to the constructor:

```python
model = KyTea(
    debug="silent",
    input_format="raw",
    word_bound=" ",
    tag_bound="/",
    elem_bound="&",
    unk_bound=" ",
    no_bound="-",
    has_bound="|",
    skip_bound="?",
    escape="\\",
)
```

The default values of `*_bound` and `escape` are described [here](https://docs.rs/kytea-sys/0.2.5/kytea_sys/ffi/defaults/index.html).

The parameter `debug` represents the log level on the FFI side: `"silent"` (*default*), `"simple"`, `"detailed"`, and `"full"`.

The list of the accepted values of `input_format` is: `"raw"` (*default*), `"full"`, `"part"`, `"tok"`, `"eda"`, and `"tags"`.
See the documentation of KyTea for detailed information.
