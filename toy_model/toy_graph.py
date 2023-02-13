from torch_geometric.data import Data

from toy_feature_gen import node_features, edge_index, edge_weights

data = Data(x=node_features, edge_index=edge_index, edge_attr=edge_weights)
