# Dockerfile for a containerised instance of the html_to_markdown command line app

FROM cgr.dev/chainguard/python:latest-dev AS dev

COPY . /source

# Install as root, but we run as nonroot
USER root
RUN pip install /source


# The final image uses the minimal Chainguard version
FROM cgr.dev/chainguard/python:latest
COPY --from=dev /usr/lib/python3.13/site-packages /usr/lib/python3.13/site-packages

ENTRYPOINT ["python", "-m", "html_to_markdown"]
