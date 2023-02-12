from collections import Counter
from enum import Enum

# Imagine we have imported nice representations of stations and lines from tubeulator
# For now we mock these with a simple dictionary of a subset of inner London stations
# which we ensure are not subject to typos etc. by keeping a unique Enum of station names


class Lines(Enum):
    Central = "red"
    Victoria = "cyan"
    Jubilee = "grey"
    Northern = "black"
    Piccadilly = "navy"
    Waterloo = "brown"


class Stations(Enum):
    BondStr = "Bond Street"
    CharingX = "Charing Cross"
    GreenPark = "Green Park"
    HPC = "Hyde Park Corner"
    LeicesterSq = "Leicester Square"
    MarbleArch = "Marble Arch"
    OxfordCirc = "Oxford Circus"
    PiccadillyCirc = "Piccadilly Circus"
    TCR = "Tottenham Court Road"
    WarrenStr = "Warren Street"
    Westminster = "Westminster"


class StationLocations(Enum):
    MarbleArch = (0.1, 0.3)
    BondStr = (0.3, 0.3)
    HPC = (0.2, 0.6)
    GreenPark = (0.3, 0.6)
    OxfordCirc = (0.5, 0.4)
    PiccadillyCirc = (0.6, 0.6)
    TCR = (0.7, 0.4)
    Westminster = (0.6, 0.9)
    LeicesterSq = (0.7, 0.6)
    CharingX = (0.7, 0.8)
    WarrenStr = (0.7, 0.0)


assert set(s.name for s in Stations) == set(s.name for s in StationLocations)

network = {
    Lines.Central: [
        Stations.MarbleArch,
        Stations.BondStr,
        Stations.OxfordCirc,
        Stations.TCR,
    ],
    Lines.Victoria: [
        Stations.WarrenStr,
        Stations.OxfordCirc,
        Stations.GreenPark,
    ],
    Lines.Jubilee: [
        Stations.BondStr,
        Stations.GreenPark,
        Stations.Westminster,
    ],
    Lines.Northern: [
        Stations.WarrenStr,
        Stations.TCR,
        Stations.LeicesterSq,
        Stations.CharingX,
    ],
    Lines.Piccadilly: [
        Stations.HPC,
        Stations.GreenPark,
        Stations.PiccadillyCirc,
        Stations.LeicesterSq,
    ],
    Lines.Waterloo: [
        Stations.OxfordCirc,
        Stations.PiccadillyCirc,
        Stations.CharingX,
    ],
}

node_names = list(sorted(station.value for station in Stations))
line_names = list(sorted(line.name for line in Lines))
idx2node = dict(enumerate(node_names))
idx2line = dict(enumerate(line_names))
node2idx = {n: i for i, n in idx2node.items()}
line2idx = {n: i for i, n in idx2line.items()}
stations_by_line = [
    *sorted(
        [(s.value, line.name) for line, stations in network.items() for s in stations]
    )
]
station_line_counter = Counter([s for s, _ in stations_by_line])
station_line_lookup = {
    station: [line for name, line in stations_by_line if name == station]
    for station in station_line_counter
}
interchanges = {
    station: [line for name, line in stations_by_line if name == station]
    for station, c in station_line_counter.items()
    if c > 1
}
station_line_idx_lookup = {
    node2idx[station]: [line2idx[line] for line in lines]
    for station, lines in station_line_lookup.items()
}
