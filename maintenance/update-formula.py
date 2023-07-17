#!/usr/bin/env python3
import json
from jinja2 import Environment, FileSystemLoader
from pathlib import Path

here = Path(__file__).parent.absolute()
target_path = here.parent.joinpath('Formulas')
env = Environment(loader=FileSystemLoader(here))


def load_formula_metadata():
    for file in here.parent.joinpath("etc", "brew", "formulae").glob("*/*/metadata.v2.json"):
        with open(file) as fd:
            yield json.load(fd)


def load_cask_metadata():
    for file in here.parent.joinpath("etc", "brew", "casks").glob("*/*/metadata.v2.json"):
        with open(file) as fd:
            yield json.load(fd)


def main():
    tmpl = env.get_template("formula.rb.j2")
    rendered = tmpl.render(casks=load_cask_metadata(), formulae=load_formula_metadata())
    with target_path.joinpath('aikido.rb').open('w') as fd:
        fd.write(rendered)


if __name__ == '__main__':
    main()
