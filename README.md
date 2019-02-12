# timekeep

website visitor logging that's nice to your visitors

stuff:

- don't track people on an individual level. respect people. maybe don't use this at all.
- visitor logging is the least important, most expendable data.
- don't add features. especially if they won't give clearly actionable information.
- don't save anything for more than 30 days. let it go.

## How to use

```sh

# Add the remote git on Heroku
heroku git:remote -a "<your app name here>"

# Use a Rust buildpack
heroku buildpacks:set https://github.com/emk/heroku-buildpack-rust

# Deploy
git push heroku master

# Scale up web process
heroku ps:scale web=1
```
