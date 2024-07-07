import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as lambda from 'aws-cdk-lib/aws-lambda';

export class PM extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);


    // output video bucket
    const outputBucket = new s3.Bucket(this, 'OutputBucket', {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    // lambda function to convert audio to video
    const audioToVideoLambda = new lambda.Function(this, 'AudioToVideoLambda', {
      runtime: lambda.Runtime.FROM_IMAGE,
      handler: lambda.Handler.FROM_IMAGE,
      code: lambda.Code.fromAssetImage('function', {
        ignoreMode: cdk.IgnoreMode.GIT,
        exclude: ['.idea', 'target']
      }),
      environment: {
        OUTPUT_BUCKET: outputBucket.bucketName,
      },
      reservedConcurrentExecutions: 10,
      timeout: cdk.Duration.minutes(15),
      memorySize: 1024 * 4,
      ephemeralStorageSize: cdk.Size.gibibytes(5),
    });
    const fnUrl = audioToVideoLambda.addFunctionUrl({
      cors: {
        allowedOrigins: ['*'],
      },
      authType: lambda.FunctionUrlAuthType.NONE
    })

    // grant permission to lambda to write to output bucket
    outputBucket.grantReadWrite(audioToVideoLambda);

    // generate lambda URL
    new cdk.CfnOutput(this, 'LambdaURL', {
      value: fnUrl.url,
    });

    // bucket
    new cdk.CfnOutput(this, 'OutputBucketName', {
      value: outputBucket.bucketName,
    });
  }
}
