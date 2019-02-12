# timekeep

website visitor logging that's nice to your visitors

stuff:

- respect people
- don't break stuff
- don't track people on an individual level. maybe don't use this at all.
- visitor logging is the least important, most expendable data.
- don't add features. especially if they won't give clearly actionable information. also because redeploying wipes the counts lol.
- don't save anything for more than 30 days. let it go.
- no cookies!

todo:

- maybe collect stats on user agents => eventually just try to estimate how many visits are bots
- consider not adding ips to the bloom if DNT=1

notes:

- uniques and new visitor counts are based on IP addresses, which may be used by more than one person, so they are an upper bound.
- if any intermediate servers ignore the cache headers and don't let the request get back to the service, those visitors will be missed.
- any browser extension might start blocking this.
- ip addresses aren't actually stored, just bloom filters tuned for 10k addresses at 3% false-positive. so, it should under-count by less than 3% if you have less than 10000 folks visiting, or by some amount more than 3% if you have more.
    - this isn't a promise or probably important, but this uncertainty does mean: "computer at ip X visited this website" might always be false, and "computer at ip X did not visit this website" also always might be false.

other notes:

- why not just use your server logs? if you have them, you don't need this. this is for eg., github pages where you don't have them.
- why is the data public? why not.
- can i use your hosted instance? sure. it's currently at https://timekeep-server.herokuapp.com/, so just add `<img src="https://timekeep-server.herokuapp.com/count.gif" style="position: absolute; left:-9999em" alt="visitor counter" aria-hidden="true" />` before your closing `</body>` tag on any page you want to count.
    - i might cut you off if you use up all the bandwidth. maybe i'll change my mind and shut it down. should be easy to host your own.
    - i don't think my websites get enough traffic to actually keep the heroku dyno alive, so it probably won't work super well for ya. or me. maybe your traffic will help :)
