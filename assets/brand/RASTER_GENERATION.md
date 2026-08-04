# Raster generation

Derived PNG assets are regenerated from the canonical SVG masters with:

```sh
python -m pip install 'CairoSVG==2.8.2' 'Pillow==12.2.0'
python scripts/generate-brand-rasters.py
```

The generator rewrites the social preview as a self-contained SVG before rasterization, so GitHub and other renderers do not depend on relative external SVG references. The Playground brand contract pins the resulting PNG hashes to reject dimension-correct but corrupted files.
