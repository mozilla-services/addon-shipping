import logging
import tempfile
import zipfile

import boto3
import marshmallow.fields

logger = logging.getLogger()
logger.setLevel(logging.INFO)

s3 = boto3.resource('s3')

class S3BucketInfo(marshmallow.Schema):
    name = marshmallow.fields.String()
    arn = marshmallow.fields.String()


class S3ObjectInfo(marshmallow.Schema):
    key = marshmallow.fields.String()


class S3Path(marshmallow.Schema):
    bucket = marshmallow.fields.Nested(S3BucketInfo)
    object = marshmallow.fields.Nested(S3ObjectInfo)


class S3Event(marshmallow.Schema):
    event_time = marshmallow.fields.String(load_from="eventTime")
    event_name = marshmallow.fields.String(load_from="eventName")
    aws_region = marshmallow.fields.String(load_from="awsRegion")
    s3 = marshmallow.fields.Nested(S3Path)


class S3BatchEvent(marshmallow.Schema):
    requests = marshmallow.fields.List(marshmallow.fields.Nested(S3Event), load_from="Records")


def handle(event, context):
    """
    Handle a sign-xpi event.
    """

    event = S3BatchEvent(strict=True).load(event).data
    responses = []
    for request in event['requests']:
        filename = request['s3']['object']['key']
        if not filename.endswith(u'.xpi'):
            responses.append({"error": "file not XPI: {}".format(filename)})
            continue

        if not request['event_name'].startswith(u'ObjectCreated'):
            responses.append({"error": "event not ObjectCreated: {}".format(request['event_name'])})
            continue

        bucket_name = request['s3']['bucket']['name']
        bucket = s3.Bucket(bucket_name)
        with tempfile.TemporaryFile() as localfile:
            bucket.download_fileobj(filename, localfile)

            xpi = zipfile.ZipFile(localfile)

            manifest_name = None
            xpi_files = xpi.namelist()
            for possible_manifest in ['install.rdf', 'manifest.json']:
                if possible_manifest in xpi_files:
                    manifest_name = possible_manifest
                    break

            if not manifest_name:
                responses.append({"error": "No manifest found in {}".format(filename)})
                break

            manifest_contents = xpi.read(manifest_name)

        logger.info("Manifest for %s: %s", filename, manifest_name)
        logger.info("%s", manifest_contents)
        # FIXME: point to some other XPI
        responses.append({"uploaded": request['s3']})

    return responses
