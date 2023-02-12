from enum import Enum

import numpy as np

# Imagine we have imported nice representations of stations and lines from tubeulator
# For now we mock these with a simple dictionary of a subset of inner London stations
# which we ensure are not subject to typos etc. by keeping a unique Enum of station names


class Lines(Enum):
    Central = "red"
    Victoria = "cyan"
    Jubilee = "grey"
    Piccadilly = "navy"
    Waterloo = "brown"


class Stations(Enum):
    BondStreet = "Bond Street"
    CharingCross = "Charing Cross"
    GreenPark = "Green Park"
    HPC = "Hyde Park Corner"
    LeicesterSquare = "Leicester Square"
    MarbleArch = "Marble Arch"
    OxfordCircus = "Oxford Circus"
    PiccadillyCircus = "Piccadilly Circus"
    TCR = "Tottenham Court Road"
    WarrenStreet = "Warren Street"
    Westminster = "Westminster"


network = {
    Lines.Central: [
        Stations.MarbleArch,
        Stations.BondStreet,
        Stations.OxfordCircus,
        Stations.TCR,
    ],
    Lines.Victoria: [
        Stations.WarrenStreet,
        Stations.OxfordCircus,
        Stations.GreenPark,
    ],
    Lines.Jubilee: [
        Stations.BondStreet,
        Stations.GreenPark,
        Stations.Westminster,
    ],
    Lines.Piccadilly: [
        Stations.HPC,
        Stations.GreenPark,
        Stations.PiccadillyCircus,
        Stations.LeicesterSquare,
    ],
    Lines.Waterloo: [
        Stations.OxfordCircus,
        Stations.PiccadillyCircus,
        Stations.CharingCross,
    ],
}
