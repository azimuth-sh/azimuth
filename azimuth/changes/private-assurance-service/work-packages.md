# Work packages: private-assurance-service

## Work package: service
Status: complete
Depends on: none
Owns: services/assurance/server, services/assurance/web, services/assurance/README.md, services/assurance/docker-compose.yml
Objective: Implement GitHub-authenticated inspection-only access to the current Assurance State Service and align image packaging and service documentation

## Work package: integration
Status: complete
Depends on: service
Owns: .dockerignore, azimuth/changes/private-assurance-service, docs/private-assurance-deployment.md, azimuth/model/framework/assurance-deployment
Objective: Integrate the independently owned Terraform repository changes with service contracts and record verified results and remaining deployment inputs

## Work package: image-publication
Status: in-progress
Depends on: service, integration
Owns: .github/workflows/publish-assurance.yml
Objective: Publish only API and Web images from an exact source revision using native AMD64 and ARM64 builds without invoking test or full release workflows
