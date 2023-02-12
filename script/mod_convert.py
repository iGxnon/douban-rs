import os
import shutil
from pathlib import Path


def mod_2018_to_2015(root_dir):
    for (root, dirs, files) in os.walk(root_dir, topdown=True):
        converts = []

        for file in files:
            striped = str(file).split('.')[0]
            if striped in dirs:
                converts.append((str(file), striped))

        for file, dirc in converts:
            print(f'move {file}')
            dirc = Path(root).joinpath(dirc)
            file = Path(root).joinpath(file)
            shutil.move(file, dirc.joinpath('mod.rs'))


def mod_2015_to_2018(root_dir):

    def move(m_file: Path):
        print(f'move {m_file}')
        shutil.move(m_file, m_file.parent.parent.joinpath(m_file.parent.name + '.rs'))

    for (root, dirs, files) in os.walk(root_dir, topdown=True):
        for d in dirs:
            m_files = [m_file for m_file in Path(root).joinpath(d).iterdir()
                       if m_file.is_file() and m_file.name == 'mod.rs']
            for m_file in m_files:
                move(m_file)


if __name__ == '__main__':
    # service codes with rust 2015 mod style might be clear
    mod_2018_to_2015('../service')
    mod_2015_to_2018('../common')
