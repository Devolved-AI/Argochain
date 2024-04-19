import toml
from collections import defaultdict

# Step 1: Read the file content
with open('Cargo.toml', 'r') as file:
    data = toml.load(file)

# Step 2: Parse the content to identify the packages
packages = data.get('dependencies', {})

# Step 3: Group the packages based on the specified criteria
# Here we group by the first letter of the package name
groups = defaultdict(list)
for package, details in packages.items():
    groups[package[0].lower()].append((package, details))

# Step 4: Sort the groups
sorted_groups = sorted(groups.items())

# Step 5: Write the sorted groups back to the file
sorted_packages = {}
for group in sorted_groups:
    for package, details in group[1]:
        sorted_packages[package] = details

data['dependencies'] = sorted_packages

with open('Cargo.toml', 'w') as file:
    toml.dump(data, file)
