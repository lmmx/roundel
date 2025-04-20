(
  # Loop over both directions
  for direction in inbound outbound
  do
    # Loop over each JSON file in that direction
    for f in routes/"$direction"/*.json
    do
      # Extract just the “bakerloo”, “central” etc. from the filename
      line="$(basename "$f" .json)"
      # Emit one object that includes the line, the direction, and the file’s contents
      echo "{\"line\":\"$line\",\"direction\":\"$direction\",\"data\":$(cat "$f")}"
    done
  done
) \
| jq -s '
    reduce .[] as $obj (
      {};
      # For each object, put the data under obj.line → obj.direction
      .[$obj.line][$obj.direction] = ($obj.data | del(.results[] ["Stations", "LineName", "IsOutboundOnly", "StopPointSequences", "OrderedLineRoutes"]))
    )
' > rail_routes.json

(
  # Loop over both directions
  for direction in inbound outbound
  do
    # Loop over each JSON file in that direction
    for f in routes/"$direction"/bus/*.json
    do
      # Extract just the “bakerloo”, “central” etc. from the filename
      line="$(basename "$f" .json)"
      # Emit one object that includes the line, the direction, and the file’s contents
      echo "{\"line\":\"$line\",\"direction\":\"$direction\",\"data\":$(cat "$f")}"
    done
  done
) \
| jq -s '
    reduce .[] as $obj (
      {};
      # For each object, put the data under obj.line → obj.direction
      .[$obj.line][$obj.direction] = ($obj.data | del(.results[] ["Stations", "LineName", "IsOutboundOnly", "StopPointSequences", "OrderedLineRoutes"]))
    )
' > bus_routes.json
