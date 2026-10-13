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
