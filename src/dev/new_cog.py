
import sys
import json
from pathlib import Path


def main():
    name = input("Enter new cog's name (PascalCase!): ")
    cogs_folder = Path(__file__).parent.parent.joinpath("cogs")
    path = cogs_folder.joinpath(f"{name.lower()}.py")
    if path.exists():
        print("This cog seems to already exist! Refusing to overwrite.")
        sys.exit(1)
    else:
        print("Creating new Python file...")
        template_content = Path(__file__).parent.joinpath("cog.template.py").read_text()
        new_cog_file = template_content.replace("__CName__", name)
        path.write_text(new_cog_file)
        print("Adding to \"cogs.json\"...")
        cogs_json_path = cogs_folder.parent.joinpath("cogs.json")
        current: list[str] = json.loads(cogs_json_path.read_text())
        current.append(f"cogs.{name.lower()}")
        cogs_json_path.write_text(json.dumps(current))
        print(f"Cog Created at {path}!")

if __name__ == "__main__":
    main()
