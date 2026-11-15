"""Build the table face from the bundled Manrope font with FontTools."""

from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

directory = Path(__file__).resolve().parent
digits = "0123456789"
with TTFont(directory / "Manrope-Regular.ttf", recalcTimestamp=False) as font:
    gsub = font["GSUB"].table
    tabular = next(
        record.Feature for record in gsub.FeatureList.FeatureRecord
        if record.FeatureTag == "tnum"
    )
    substitutions = {
        source: target
        for index in tabular.LookupListIndex
        for table in gsub.LookupList.Lookup[index].SubTable
        for source, target in table.mapping.items()
    }
    cmap = font.getBestCmap()
    figures = {point: substitutions[cmap[point]] for point in map(ord, digits)}
    for table in font["cmap"].tables:
        if table.isUnicode() and table.format != 14:
            table.cmap.update(figures)

    names = {
        1: "Muzo Table Text",
        3: "MuzoTableText-Regular",
        4: "Muzo Table Text Regular",
        6: "MuzoTableText-Regular",
        16: "Muzo Table Text",
    }
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID]

    options = subset.Options(name_IDs=[0, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17])
    table_face = subset.Subsetter(options=options)
    table_face.populate(text=digits + ": plays")
    table_face.subset(font)
    font.save(directory / "MuzoTableText-Regular.ttf")
