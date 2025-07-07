# PyInstaller hook to include the google namespace package
from PyInstaller.utils.hooks import collect_submodules
hiddenimports = collect_submodules('google') 