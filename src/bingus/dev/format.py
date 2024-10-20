import os
import pathlib


def main():
    src = pathlib.Path(__file__).parent.parent.parent
    print(f"Running black on src: {src}")
    os.system(f"black {src.absolute()}")


if __name__ == "__main__":
    main()
