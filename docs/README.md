To render schema as images, you need the [`mmdc`](https://github.com/mermaid-js/mermaid-cli) compiler.  Either install it locally or run with Docker:

```sh
# local installation
make
# docker installation
make MMDC="docker run --rm -u $(id -u):$(id -g) -v .:/data minlag/mermaid-cli"
# at 42 on fedora using podman emulation
make MMDC="docker run --rm -u $(id -u):$(id -g) --userns=keep-id:uid=$(id -u),gid=$(id -g) -v .:/data minlag/mermaid-cli"
```
