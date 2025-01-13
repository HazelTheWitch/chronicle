# chronicle

Art archival and tagging.

## Installation

Head over to [releases](https://github.com/HazelTheWitch/chronicle/releases) and download the prebuilt binaries and place them on your path.
Additionally there is a shell/powershell script available to install it automatically.

Windows users can download the msi installer instead.

## Importers

Currently the list of supported websites for import is:
- bsky
- tumblr

I am looking to expand this list to include:
- twitter
- pixiv

If there is a site you would like to see added please [make an issue](https://github.com/HazelTheWitch/chronicle/issues/new) to let me know.

### Importer Specific Setup

#### Bsky

1. Run `chronicle service login bsky`
1. When prompted for `bsky-identifier` enter your Bsky username.
1. When prompted for `bsky-password` enter your Bsky password.

#### Tumblr

1. Go to [the Tumblr developer application portal](https://www.tumblr.com/oauth/apps) and register an application.
1. Fill anything in for your application's name, website, description, contact email and callback url.
1. For "Oauth2 redirect URLs" use `http://localhost:5001/oauth/redirect/tumblr`
1. Run `chronicle service login tumblr`
1. When prompted for `tumblr-consumer` enter the string of numbers and letters next to "OAuth Consumer Key" on your newly made application.
1. When prompted for `tumblr-secret` enter the string of numbers and letters next to "Secret Key" after you click "Show secret key".
1. When you import works from Tumblr you will be prompted to login and authorize the application you just made with your Tumblr account, once you've done that you should see the message "successfully logged in".
