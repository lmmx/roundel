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
# N.B. 'global' node idx is not line specific
node2idx_global = {n: i for i, n in idx2node.items()}
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
station_line_uniq_global_lut = dict(
    enumerate(
        [
            (station, line)
            for station, lines in station_line_lookup.items()
            for line in sorted(lines, key=lambda k: line2idx[k])
        ]
    )
)
station_line_uniq_global_idx_lut = {
    station_line_idx: (node2idx_global[global_station_name], line2idx[line_name])
    for station_line_idx, (
        global_station_name,
        line_name,
    ) in station_line_uniq_global_lut.items()
}
station_line_uniq_names = [
    f"{station} ({line})" for station, line in station_line_uniq_global_lut.values()
]
station_line_uniq_name_lut = dict(enumerate(station_line_uniq_names))

# >>> from pprint import pprint; pp = lambda x: pprint(x, sort_dicts=False)
# >>> pp(station_line_uniq_global_lut)
# {0: ('Bond Street', 'Central'),
#  1: ('Bond Street', 'Jubilee'),
#  2: ('Charing Cross', 'Northern'),
#  3: ('Charing Cross', 'Waterloo'),
#  4: ('Green Park', 'Jubilee'),
#  5: ('Green Park', 'Piccadilly'),
#  6: ('Green Park', 'Victoria'),
#  7: ('Hyde Park Corner', 'Piccadilly'),
#  8: ('Leicester Square', 'Northern'),
#  9: ('Leicester Square', 'Piccadilly'),
#  10: ('Marble Arch', 'Central'),
#  11: ('Oxford Circus', 'Central'),
#  12: ('Oxford Circus', 'Victoria'),
#  13: ('Oxford Circus', 'Waterloo'),
#  14: ('Piccadilly Circus', 'Piccadilly'),
#  15: ('Piccadilly Circus', 'Waterloo'),
#  16: ('Tottenham Court Road', 'Central'),
#  17: ('Tottenham Court Road', 'Northern'),
#  18: ('Warren Street', 'Northern'),
#  19: ('Warren Street', 'Victoria'),
#  20: ('Westminster', 'Jubilee')}
# >>> pp(station_line_uniq_global_idx_lut)
# {0: (0, 0),
#  1: (0, 1),
#  2: (1, 2),
#  3: (1, 5),
#  4: (2, 1),
#  5: (2, 3),
#  6: (2, 4),
#  7: (3, 3),
#  8: (4, 2),
#  9: (4, 3),
#  10: (5, 0),
#  11: (6, 0),
#  12: (6, 4),
#  13: (6, 5),
#  14: (7, 3),
#  15: (7, 5),
#  16: (8, 0),
#  17: (8, 2),
#  18: (9, 2),
#  19: (9, 4),
#  20: (10, 1)}

nodeidx2latlong_global = {
    idx: StationLocations[Stations(station_name).name].value
    for station_name, idx in node2idx_global.items()
}

node_features = {
    stationline_idx: [*nodeidx2latlong_global[station_idx], line_idx]
    for stationline_idx, (
        station_idx,
        line_idx,
    ) in station_line_uniq_global_idx_lut.items()
}

# >>> pp(node_features)
# {0: [0.3, 0.3, 0],
#  1: [0.3, 0.3, 1],
#  2: [0.7, 0.8, 2],
#  3: [0.7, 0.8, 5],
#  4: [0.3, 0.6, 1],
#  5: [0.3, 0.6, 3],
#  6: [0.3, 0.6, 4],
#  7: [0.2, 0.6, 3],
#  8: [0.7, 0.6, 2],
#  9: [0.7, 0.6, 3],
#  10: [0.1, 0.3, 0],
#  11: [0.5, 0.4, 0],
#  12: [0.5, 0.4, 4],
#  13: [0.5, 0.4, 5],
#  14: [0.6, 0.6, 3],
#  15: [0.6, 0.6, 5],
#  16: [0.7, 0.4, 0],
#  17: [0.7, 0.4, 2],
#  18: [0.7, 0.0, 2],
#  19: [0.7, 0.0, 4],
#  20: [0.6, 0.9, 1]}
