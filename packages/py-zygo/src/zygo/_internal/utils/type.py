# Recursively describes a type representable by JSON (dict, list, str, int, float, bool, None)
Jsonable = (
    str | int | float | bool | None | list["Jsonable"] | dict[str, "Jsonable"] | None
)
