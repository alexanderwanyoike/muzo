"""Build the table face from the bundled Manrope font with FontTools."""

from enum import IntEnum
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont


class NameID(IntEnum):
    COPYRIGHT = 0
    FAMILY = 1
    SUBFAMILY = 2
    UNIQUE_IDENTIFIER = 3
    FULL_NAME = 4
    VERSION = 5
    POSTSCRIPT = 6
    LICENSE = 13
    LICENSE_URL = 14
    TYPOGRAPHIC_FAMILY = 16
    TYPOGRAPHIC_SUBFAMILY = 17


UNICODE_VARIATION_SEQUENCES = 14

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
        if table.isUnicode() and table.format != UNICODE_VARIATION_SEQUENCES:
            table.cmap.update(figures)

    names = {
        NameID.FAMILY: "Muzo Table Text",
        NameID.UNIQUE_IDENTIFIER: "MuzoTableText-Regular",
        NameID.FULL_NAME: "Muzo Table Text Regular",
        NameID.POSTSCRIPT: "MuzoTableText-Regular",
        NameID.TYPOGRAPHIC_FAMILY: "Muzo Table Text",
    }
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID]

    options = subset.Options(name_IDs=[int(name) for name in NameID])
    table_face = subset.Subsetter(options=options)
    table_face.populate(text=digits + ": plays tracks")
    table_face.subset(font)
    font.save(directory / "MuzoTableText-Regular.ttf")
