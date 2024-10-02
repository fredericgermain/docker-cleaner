# Docker Cleaner

Docker Cleaner is a Rust-based command-line tool for analyzing and cleaning up dangling Docker data. It provides a comprehensive view of the Docker data structure and allows for the removal of unused components.

## Warning

This tool should be used with caution, especially when deleting objects. It's recommended to stop the Docker daemon and create a backup before performing any cleanup operations.
This completes the code for all the files in the project. The implementation provides a foundation for analyzing and cleaning up Docker data, but there are still areas that could be improved or expanded, such as error handling, more detailed analysis of dependencies, and additional cleanup options.

## Categories

The tool analyzes and displays the following categories of Docker objects:

1. **Upper level nodes**:
   - **Image Repositories**: Represents Docker images with their tags.
   - **Containers**: Represents Docker containers.
2. **Dangling Objects**:
   - **Overlay2 Layers**: Filesystem layers used by Docker's overlay2 storage driver.
   - **Image Layers**: Intermediate layers that make up Docker images.
   - **Image Contents**: Actual image data and metadata.
3. **Missing nodes**:

## Graph Logic

The tool builds a graph of dependencies between different Docker objects:

- **Overlay2Node**: Represents an overlay2 filesystem layer. It may depend on other Overlay2Nodes (lower layers).
- **ImageLayerNode**: Represents an image layer. It depends on an Overlay2Node.
- **ImageContentNode**: Represents the content of an image. It depends on multiple ImageLayerNodes.
- **ImageRepoNode**: Represents an image in a repository. It depends on an ImageContentNode.
- **ContainerNode**: Represents a container. It depends on an ImageContentNode.

The graph is constructed by analyzing the Docker data directory structure and the contents of various metadata files. Dependencies are established based on the relationships between these objects in the Docker ecosystem.

## Usage
docker-cleaner [OPTIONS]
OPTIONS:
-b, --base <PATH>    Set the base directory (default: /var/lib/docker)

## Example of /var/lib/docker corruption / dangling files

[Failed to register layer: no such file or directory](use_cases/docker_x.x.x_failed_to_register_layer.md)

## TODO

- [Leftover Dandling nodes Overlay2](todo/boltdb_buildkit_deps.md)
- Just let every object be explored, not only the dangling ones
- back/quit on escape
- pop node layer automatically when they are deleted
- refresh layer automatically when they are active again (list of node by type, when we just deleted one)
