# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[('env.example', '.'), ('vocabulary_notebook.md', '.')],
    hiddenimports=[
        'click.core', 'click.decorators', 'click.types', 'click.utils',
        'rich.console', 'rich.panel', 'rich.text', 'rich.markdown',
        'google.genai', 'google.genai.types',
        'dotenv',
        'git',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # GUI frameworks
        'tkinter', 'PyQt5', 'PyQt6', 'PySide2', 'PySide6', 'wx', 'kivy',
        # Data science libraries
        'matplotlib', 'numpy', 'pandas', 'scipy', 'seaborn', 'plotly',
        # Image processing
        'PIL', 'Pillow', 'opencv', 'cv2',
        # Web frameworks
        'flask', 'django', 'fastapi', 'tornado', 'aiohttp',
        # Database
        'sqlite3', 'mysql', 'postgresql', 'sqlalchemy', 'pymongo',
        # Testing
        'pytest', 'unittest', 'doctest', 'nose', 'tox',
        # Documentation
        'sphinx', 'docutils', 'jinja2', 'markdown',
        # Development tools
        'setuptools', 'distutils', 'wheel', 'pip', 'virtualenv', 'venv',
        'black', 'flake8', 'pylint', 'mypy',
        # Jupyter/IPython
        'IPython', 'jupyter', 'notebook', 'ipykernel',
        # Other heavy libraries
        'tensorflow', 'torch', 'sklearn', 'scikit-learn',
        'boto3', 'azure', 'google.cloud',
        'celery', 'redis', 'rabbitmq',
        'cryptography',
        'aiofiles',
        'yaml', 'toml', 'xml', 'json5',
        'lxml', 'beautifulsoup4', 'selenium',
        'pytz', 'dateutil', 'arrow',
        'psutil', 'pywin32', 'win32api',
    ],
    noarchive=False,
    optimize=2,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='word4you',
    debug=False,
    bootloader_ignore_signals=False,
    strip=True,  # Strip debug symbols to reduce size
    upx=False,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
