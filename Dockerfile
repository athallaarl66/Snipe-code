# The release workflow unpacks the published Linux archive into this build context, so
# the image carries exactly the binary the release ships, not a second build of it.
FROM gcr.io/distroless/cc-debian13:nonroot

ARG VERSION=dev
ARG SOURCE_REVISION=unknown
LABEL org.opencontainers.image.source="https://github.com/athallaarl66/Snipe-code" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${SOURCE_REVISION}"

COPY snipe-code /usr/local/bin/snipe-code

ENTRYPOINT ["snipe-code"]
