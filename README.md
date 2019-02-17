# timekeep

website visitor logging that's nice to your visitors

stuff:

- respect people
- don't break things
- don't track people on an individual level. maybe don't use this at all.
- visitor logging is the least important, most expendable data.
- don't add features. especially if they won't give clearly actionable information. also because redeploying wipes the counts lol.
- don't save anything for more than 30 days. let it go.
- no cookies!

todo:

- [ ] don't add ips to the bloom filters if DNT=1
- [ ] add dnt-policy.txt
- [ ] generate little svg graphs for timeseries
- [ ] allow customizing timezone

notes:

- uniques and new visitor counts are based on IP addresses, which may be used by more than one person, so they are an upper bound.
- if any intermediate servers ignore the cache headers and don't let the request get back to the service, those visitors will be missed.
- any browser extension might start blocking this.
- host and path are parsed from the Referer header -- browsers that don't include this won't be counted.
- ip addresses aren't actually stored, just bloom filters tuned for 10k addresses at 3% false-positive. so, it should under-count by less than 3% if you have less than 10000 folks visiting, or by some amount more than 3% if you have more.
    - this isn't a promise or probably important, but this uncertainty does mean: "computer at ip X visited this website" might always be false, and "computer at ip X did not visit this website" also always might be false.

other notes:

- why not just use your server logs? if you have them, you don't need this. this is for eg., github pages where you don't have them.
- why is the data public? why not. if you use nginx or another reverse proxy, you could add basic auth there.
- how to use: add `<img src="https://<DOMAIN THIS RUNS AT>/count.gif" style="position: absolute; left:-9999em" alt="visitor counter" aria-hidden="true" />` before your closing `</body>` tag on any page you want to count.
- running it at a subdomain of the site your counting is probably a good idea.
- you can add the counter on as many domains as you like, but privacy tools like Privacy Badger may start auto-blocking for visitors.
- heroku doesn't work for this, since no service is allowed to survive more than 24 hours at a stretch, and timekeep stores all data in memory.

even more notes:
- cloudflare used to report ~2000 unique visitors each month for one website I run. The numbers I get from timekeep on the same site are more like ~100 uniques per month. I believe the difference is bots.
