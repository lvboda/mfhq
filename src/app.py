import argparse
import importlib

SCRIPTS = {
    "yingxionggu": "yingxionggu",
    "pangpang": "pangpang",
    "controller": "controller",
}


def main():
    parser = argparse.ArgumentParser(prog="mfhq")
    parser.add_argument("script", choices=sorted(SCRIPTS), help="script to run")
    args = parser.parse_args()

    module = importlib.import_module(SCRIPTS[args.script])
    module.main()


if __name__ == "__main__":
    main()
