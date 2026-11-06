"""Build InterTabularDigits.ttf: Inter's tabular (tnum) digits as the default glyphs.

egui has no switch for OpenType features, so beau bakes `tnum` into a tiny
digits-only font and puts it first in each family's fallback chain. Digits get
fixed widths; everything else comes from InterVariable.ttf. Variable axes are kept.

Usage: python tabular_digits.py InterVariable.ttf InterTabularDigits.ttf
Requires: fonttools
"""
import sys
from fontTools.ttLib import TTFont
from fontTools import subset

src, out = sys.argv[1], sys.argv[2]
f = TTFont(src)
cmap = f.getBestCmap()
gsub = f["GSUB"].table
# Collect single substitutions from the 'tnum' feature.
mapping = {}
for fr in gsub.FeatureList.FeatureRecord:
    if fr.FeatureTag != "tnum":
        continue
    for li in fr.Feature.LookupListIndex:
        lookup = gsub.LookupList.Lookup[li]
        for st in lookup.SubTable:
            if hasattr(st, "ExtSubTable"):
                st = st.ExtSubTable
            if hasattr(st, "mapping"):
                mapping.update(st.mapping)
digits = [ord(c) for c in "0123456789"]
for t in f["cmap"].tables:
    for cp in digits:
        if cp in t.cmap and t.cmap[cp] in mapping:
            t.cmap[cp] = mapping[t.cmap[cp]]
print({chr(cp): f.getBestCmap()[cp] for cp in digits})
opts = subset.Options()
opts.layout_features = []
opts.name_IDs = ["*"]
opts.notdef_outline = True
s = subset.Subsetter(opts)
s.populate(unicodes=digits)
s.subset(f)
f.save(out)
