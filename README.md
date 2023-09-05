# Workflow
Automate your coding workflow

# Notes

* Jira REST doc https://docs.atlassian.com/software/jira/docs/api/REST/9.7.2/
* Create Jira Token https://id.atlassian.com/manage-profile/security/api-tokens
* Search tickets : `curl -v 'https://<domain>.atlassian.net/rest/api/2/search?jql=project="XXXX"' --user $JIRA_USER:$JIRA_TOKEN`
* Get ticket by id `https://<domain>.atlassian.net/rest/api/2/issue/884748`
* Get ticket by key `curl --user $JIRA_USER:$JIRA_TOKEN https://<domain>.atlassian.net/rest/api/2/issue/XXXX-976`