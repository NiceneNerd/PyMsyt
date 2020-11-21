# PyMsyt

MSBT editing in Python using MSYT

PyMsyt is a Python library wrapping the [MSYT project](https://github.com/ascclemens/msyt) in Rust.
It uses a [custom fork](https://github.com/NiceneNerd/msyt) built as both a library and application.
PyMsyt supports reading and writing MSBT files in binary, YAML, and JSON formats. Basic 
documentation:

## API

### Class `pymsyt.Msbt`

Class representing an MSBT file. This is just a thin wrapper over the `Msyt` type from
the Rust MSYT project providing a few convenient methods for Python use. Note that
manipulating the contents of an MSBT in code can only be done staightforwardly by converting
with `to_dict()` and `from_dict()`. This class cannot be directly instantiated. Instead,
create it through static parsing methods. Example use:

```python
from pymsyt import Msbt
data = open("ArmorHead.msbt", "rb").read()
msbt = Msbt.from_binary(data)
msyt_text = msbt.to_yaml() # Convert MSBT to MSYT YAML
json_text = msbt.to_json() # Convert MSBT to JSON
msbt_dict = msbt.to_dict() # Convert to an editable Python dictionary
for entry, contents in msbt_dict["entries"].items() # Iterate MSBT text entries
    print(f"{entry} = {contents}")
msbt_dict["entries"]["Armor_999_Head"] = { # Adding a new text entry
    "contents": [{"text":"Some new helmet"}]
}
open("ArmorHead.msbt", "wb").write( # Saving modified file
    Msbt.from_dict(msbt_dict).to_binary(big_endian=True)
)
```

#### Methods defined here:

> **to_binary(big_endian: bool) -> bytes**

Serializes this MSBT file to bytes.

> **to_dict() -> dict**

Converts the MSBT contents to a Python dict.

> **to_json() -> str**

Generates a JSON representation of this MSBT file.

> **to_yaml() -> str**

Generates a YAML representation of this MSBT file.

> **from_binary(data: BytesLike) -> Msbt**

Parses an MSBT file from a byteslike object

> **from_dict(dict: dict) -> Msbt**

Parses an MSBT file from a Python dictionary.

> **from_json(json: str) -> Msbt**

Parses an MSBT file from a JSON representation.

> **from_yaml(yaml: str) -> Msbt**

Parses an MSBT file from a YAML representation.

### Class `pymsyt.MsytError`

Generic exception thrown for all errors with this library.