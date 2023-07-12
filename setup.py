#!/usr/bin/env python
# -*- coding: utf-8 -*-
# <aikido - set of tools for the self-defense of MacOS users>
# Copyright (C) <2023>  Gabriel Falcão <gabriel@nacaolivre.org>
#
# Permission is hereby granted, free of charge, to any person
# obtaining a copy of this software and associated documentation
# files (the "Software"), to deal in the Software without
# restriction, including without limitation the rights to use,
# copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the
# Software is furnished to do so, subject to the following
# conditions:
#
# The above copyright notice and this permission notice shall be
# included in all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
# OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
# HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
# WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
# FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
# OTHER DEALINGS IN THE SOFTWARE.

import ast
import os
import re
from setuptools import setup, find_packages


class VersionFinder(ast.NodeVisitor):
    def __init__(self):
        self.version = None

    def visit_Assign(self, node):
        if node.targets[0].id == 'version':
            self.version = node.value.s


def read_version():
    """Read version from aikido/version.py without loading any files"""
    finder = VersionFinder()
    finder.visit(ast.parse(local_file('src', 'version.py')))
    return finder.version


def parse_requirements(path: str):
    """Rudimentary parser for a `requirements.txt` file

    We just want to separate regular packages from links to pass them to the
    `install_requires` and `dependency_links` params of the `setup()`
    function properly.
    """
    try:
        requirements = map(str.strip, local_file(path).splitlines())
    except IOError:
        raise RuntimeError(f"Couldn't find the `{path}' file :(")

    links = []
    pkgs = []
    for req in requirements:
        if not req:
            continue
        if 'http:' in req or 'https:' in req:
            links.append(req)
            name, version = re.findall("\#egg=([^\-]+)-(.+$)", req)[0]
            pkgs.append('{0}=={1}'.format(name, version))
        else:
            pkgs.append(req.replace('==', '>='))

    return pkgs, links


def local_file(*f):
    try:
        return open(os.path.join(os.path.dirname(__file__), *f)).read()
    except:
        return ''

install_requires, dependency_links = \
    parse_requirements('requirements.txt')


if __name__ == '__main__':
    setup(
        name="ki-aikido",
        version=read_version(),
        description="Aikido is a set of tools for the self-defense of MacOS users",
        long_description=local_file('README.rst'),
        long_description_content_type="text/x-rst",
        author='Gabriel Falcao',
        author_email='gabriel@nacaolivre.org',
        url='https://github.com/gabrielfalcao/aikido',
        packages=find_packages(exclude=['*tests*']),
        install_requires=install_requires,
        dependency_links=dependency_links,
        include_package_data=True,
        entry_points={
            'console_scripts': [
                'aikido = src.cli:mate',
            ]
        },
        classifiers=[
            'Development Status :: 4 - Beta',
            'Intended Audience :: Customer Service',
            'Intended Audience :: Developers',
            'Intended Audience :: Education',
            'Intended Audience :: End Users/Desktop',
            'Intended Audience :: Science/Research',
            'Intended Audience :: System Administrators',
            'Intended Audience :: Telecommunications Industry',
            'License :: OSI Approved :: GNU General Public License v3 or later (GPLv3+)',
            'Operating System :: MacOS :: MacOS X',
            'Operating System :: MacOS',
            'Operating System :: POSIX',
            'Programming Language :: Other Scripting Engines',
            'Programming Language :: Python :: 3.7',
            'Programming Language :: Python :: 3.8',
            'Programming Language :: Python',
            'Programming Language :: Ruby',
            'Programming Language :: Rust',
            'Topic :: Artistic Software',
            'Topic :: Communications',
            'Topic :: Home Automation',
            'Topic :: Internet',
            'Topic :: Software Development :: Build Tools',
            'Topic :: Software Development :: Debuggers',
            'Topic :: Software Development :: Documentation',
            'Topic :: Software Development :: Interpreters',
            'Topic :: Software Development :: Libraries',
            'Topic :: Software Development',
        ],
    )
