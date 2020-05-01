from setuptools import setup
import os

if os.name == 'posix':
    os.chmod('pymsyt/bin/msyt', int('755', 8))

with open("docs/README.md", "r") as readme:
    long_description = readme.read()

setup(
    name='PyMsyt',
    version='0.1.5',
    author='NiceneNerd',
    author_email='macadamiadaze@gmail.com',
    description='Quick and dirty wrapper for MSYT',
    long_description=long_description,
    long_description_content_type='text/markdown',
    url='https://github.com/NiceneNerd/PyMsyt',
    include_package_data=True,
    packages=['pymsyt'],
    classifiers=[
        'Development Status :: 3 - Alpha',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3 :: Only'
    ],
    python_requires='>=3.7',
    install_requires=[
        'pyYaml>=5.1.1',
    ]
)
