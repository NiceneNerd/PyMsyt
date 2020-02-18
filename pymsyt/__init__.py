""" Provides a simple interface for using MSYT to parse, create, or convert MSBT and MSYT files """
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Union
import zlib
import yaml

EXEC_DIR = Path(os.path.dirname(os.path.realpath(__file__)))
_debug = False

if os.name == 'posix':
    MSYT_PATH = EXEC_DIR / 'bin' / 'msyt'
    CREATIONFLAGS = False
elif os.name == 'nt':
    MSYT_PATH = EXEC_DIR / 'bin' / 'msyt.exe'
    CREATIONFLAGS = True
else:
    raise RuntimeError('Only POSIX-based or Windows systems are supported.')

def parse_msbt(file: Union[str, Path]) -> {}:
    """Parses an MSBT files as an MSYT and returns a dict of the YAML contents.

    :return: Returns a dict of the MSBT contents represented in MSYT format.
    :rtype: dict
    """
    if isinstance(file, str):
        file = Path(file)
    temp_file = tempfile.NamedTemporaryFile(suffix='.msyt', delete=False)
    tmp = Path(temp_file.name)
    temp_file.close()
    if tmp.exists():
        tmp.unlink()
    if export(file, tmp):
        with tmp.open('r', encoding='utf-8') as tmp_file:
            parsed = yaml.safe_load(tmp_file)
        tmp.unlink()
        return parsed
    else:
        raise RuntimeError('MSYT failed to convert the MSBT file.')


def write_msbt(text_data: dict, output_file: Union[str, Path], platform: str = 'wiiu'):
    """Creates an MSBT file from an MSYT-formatted text data dict.

    :param text_data: The text data to write to MSBT
    :type text_data: dict
    :param output_file: The path to write the MSBT file
    :type output_file: Union[str, Path]
    :param platform: The target platform for the MSBT, either 'wiiu' or 'switch', defaults to 'wiiu'
    :type platform: str, optional
    :raises RuntimeError: Raises a `RuntimeError` if for any reason MSYT fails to create the MSBT
    """
    if isinstance(output_file, str):
        output_file = Path(output_file)
    temp_file = tempfile.NamedTemporaryFile(suffix='.msyt', delete=False)
    tmp = Path(temp_file.name)
    temp_file.close()
    with tmp.open('w', encoding='utf-8') as tmp_file:
        yaml.safe_dump(text_data, tmp_file, allow_unicode=True, encoding='utf-8')
    if create(tmp, output_file, platform=platform):
        tmp.unlink()
    else:
        raise RuntimeError('MSYT was unable to create an MSBT from the given text data.')


def export(msbt_input: Union[str, Path], msyt_output: Union[str, Path]) -> bool:
    """Exports the contents of an MSBT file in MSYT format.

    :param msbt_input: The MSBT source path, can be either a single MSBT file or a directory
    containing any number of MSBT files
    :type msbt_input: Union[str, Path]
    :param msyt_output: The MSYT destination path, can be either a single MSYT file or a directory
    to fill with multiple MSYT files
    :type msyt_output: Union[str, Path]
    :return: Returns True if the output is created successfully, otherwise False
    :rtype: bool
    """
    if isinstance(msbt_input, str):
        msbt_input = Path(msbt_input)
    if isinstance(msyt_output, str):
        msyt_output = Path(msyt_output)
    msyt_output.parent.mkdir(parents=True, exist_ok=True)
    args = [
        str(MSYT_PATH),
        'export',
        str(msbt_input.resolve()),
        '-o',
        str(msyt_output.with_suffix('').resolve())
    ]
    if msbt_input.is_dir():
        args.insert(2, '-d')
    if CREATIONFLAGS:
        subprocess.call(
            args,
            creationflags=subprocess.CREATE_NO_WINDOW if not _debug else 0,
            stderr=subprocess.DEVNULL if not _debug else sys.__stdout__,
            stdout=subprocess.DEVNULL if not _debug else sys.__stdout__
        )
    else:
        subprocess.call(
            args,
            stderr=subprocess.DEVNULL if not _debug else sys.__stdout__,
            stdout=subprocess.DEVNULL if not _debug else sys.__stdout__
        )
    return msyt_output.exists()

def create(msyt_input: Union[str, Path], msbt_ouput: Union[str, Path],
           platform: str = 'wiiu') -> bool:
    """Creates MSBT files from MSYT files

    :param msyt_input: The MSYT source path, can be either a single MSYT file or a directory
    containing any number of MSYT files
    :type msyt_input: Union[str, Path]
    :param msbt_ouput: The MSBT destination path, can be either a single MSBT file or a directory
    to fill with multiple MSBT files
    :type msbt_ouput: Union[str, Path]
    :param platform: The target platform for the MSBT, either 'wiiu' or 'switch', defaults to 'wiiu'
    :type platform: str, optional
    :return: Returns True if the output is created successfully, otherwise False
    :rtype: bool
    """
    if isinstance(msyt_input, str):
        msyt_input = Path(msyt_input)
    if isinstance(msbt_ouput, str):
        msbt_ouput = Path(msbt_ouput)
    if platform not in ['wiiu', 'switch']:
        raise ValueError(f'Invalid platform "{platform}". Only "wiiu" and "switch" are supported.')

    msbt_ouput.parent.mkdir(parents=True, exist_ok=True)
    args = [
        str(MSYT_PATH),
        'create',
        str(msyt_input.resolve()),
        '-p',
        platform,
        '-o',
        str(msbt_ouput.with_suffix('').resolve())
    ]
    if msyt_input.is_dir():
        args.insert(2, '-d')
    if CREATIONFLAGS:
        subprocess.call(
            args,
            creationflags=subprocess.CREATE_NO_WINDOW if not _debug else 0,
            stderr=subprocess.DEVNULL if not _debug else sys.__stdout__,
            stdout=subprocess.DEVNULL if not _debug else sys.__stdout__
        )
    else:
        subprocess.call(
            args,
            stderr=subprocess.DEVNULL if not _debug else sys.__stdout__,
            stdout=subprocess.DEVNULL if not _debug else sys.__stdout__
        )
    if msbt_ouput.is_file() and msbt_ouput.with_suffix('').exists():
        shutil.rmtree(msbt_ouput.with_suffix(''))
    return msbt_ouput.exists()
