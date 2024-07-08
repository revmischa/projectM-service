# ProjectM Music Visualization Service

This is a web service designed to run on AWS to convert audio files to video files by
running the audio data through the [projectM music visualizer](https://github.com/projectM-visualizer/projectm).

It makes use of the [projectM gstreamer plugin](https://github.com/projectM-visualizer/gst-projectm).

## Quickstart

Configure some AWS credentials.

```shell
npm i -g pnpm aws-cdk
pnpm install
cdk deploy
```

## Useful commands

* `pnpm exec cdk deploy`  deploy this stack to your default AWS account/region
