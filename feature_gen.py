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


class StationLocations(Enum):
    MarbleArch = (0.1, 0.3)
    BondStreet = (0.3, 0.3)
    HPC = (0.2, 0.6)
    GreenPark = (0.3, 0.6)
    OxfordCircus = (0.5, 0.4)
    PiccadillyCircus = (0.6, 0.6)
    TCR = (0.7, 0.4)
    Westminster = (0.6, 0.9)
    LeicesterSquare = (0.7, 0.6)
    CharingCross = (0.7, 0.8)
    WarrenStreet = (0.7, 0.0)


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

node_names = list(sorted(s.value for s in Stations))
node_idx = dict(enumerate(node_names))
