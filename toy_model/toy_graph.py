import torch
import torch.nn as nn
import torch.nn.functional as F
from torch_geometric.data import Data
from torch_geometric.nn import GCNConv

# from toy_feature_gen import edge_index, edge_weights, node_features

# Make it even simpler: 4 nodes, 6 bidir. consecutive connections
edge_index = torch.tensor([[0, 1, 2, 1, 2, 3], [1, 2, 1, 0, 3, 2]])
edge_weights = torch.tensor([5, 10, 10, 5, 12, 12])
node_features = torch.tensor([[0.0, 0.0], [0.4, 0.6], [0.5, 0.8], [1.0, 1.0]])

data = Data(x=node_features, edge_index=edge_index, edge_attr=edge_weights)
assert data.validate(raise_on_error=True)  # edge_index indices are in range(num_nodes)
assert not data.has_self_loops()  # You never spend time travelling to stay in one place
assert not data.has_isolated_nodes()  # You can always move around on the tube
assert data.is_undirected()  # You can travel either way on the tube


class JourneyTimeEstimator(nn.Module):
    x_masked = 2  # Only train on the first 2 features, lat. and long.

    def __init__(self, in_channels, hidden_channels, out_channels):
        super(JourneyTimeEstimator, self).__init__()
        self.conv1 = GCNConv(in_channels, hidden_channels)
        self.conv2 = GCNConv(hidden_channels, out_channels)

    def forward(self, x, edge_index):
        x = self.conv1(x, edge_index)
        x = F.relu(x)
        x = self.conv2(x, edge_index)
        # Loss does not decrease if softmax is used
        return x  # F.log_softmax(x, dim=1)


model = JourneyTimeEstimator(data.num_node_features, 32, 1)
optimiser = torch.optim.Adam(model.parameters(), lr=0.01)
criterion = nn.MSELoss()

for epoch in range(100):
    optimiser.zero_grad()
    # The output is meant to be the journey time prediction
    out = model(data.x, data.edge_index)
    if epoch < 2:
        print("The model ran")
    loss = criterion(out, data.edge_attr.float())
    if epoch < 2:
        print("The loss ran")
    loss.backward()
    optimiser.step()
    if (epoch + 1) % 10 == 0:
        print(f"Epoch {epoch + 1}/100: Loss = {loss.item()}")
