# roundel

Modeling journey time on the London tube network using graph representation learning.

## Toy dataset creation

To begin work on this, I'm using a toy dataset for the following subset of the tube network
(the stations not greyed out).

![](assets/tube-subnet.png)

## Approach

- The [`tubeulator`][tubeulator] library is used for data download
  - It reads the TfL API to get names and lat./long. of stations
- Stations serving multiple lines are duplicated by defining a station as line-specific
  - When encoding as a node feature, we use a single number to indicate the line
  - We represent line changes using a transfer edge between the different lines at
    the same station
- (WIP)

[tubeulator]: https://github.com/lmmx/tubeulator
