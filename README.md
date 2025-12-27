# Tunnel Tank Tournament

Or 'tunnel' for short. A remake of the 1991 DOS game "Tunneler", now with
online multiplayer! Not endorsed by the original creators!

See it in action here:

**[Play Tunnel Tank Tournament](tunnel.remcokranenburg.com)**

The matchmaking is very simple: every two people clicking on the link will be
matched together.

## Run it locally

If you want to run it locally, first make sure you have the right tooling:

```
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Start local dev server:

```
trunk serve --open
```

Now visit http://localhost:8080 in two browser tabs. Both tabs must be visible
at the same time, because your browser severely slows down the game if the tab
is in the background.

## Contributing

This is a personal hobby project, but I may accept PRs if they are in the
spirit of the project. Obviously, I don't want to cause trouble for the
original creators; this is a project of homage to the classics. So, there are a
few rules:

1. Copyright: no copying of code or art assets: it's all self-made
2. Trademarks: 100% clarity that this is not the original game, but a remake
   not endorsed by the original creators
3. Patents: the original game must be older than 20 years

It *is* allowed to faithfully recreate the game mechanics, because the rules of
a game cannot be owned by anyone.

Any contribution will be under the licence of the AGPL-3.0-or-later, unless
otherwise specified.

## License

Copyright 2025 Remco Kranenburg <remco@burgsoft.nl>

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

SPDX-License-Identifier: AGPL-3.0-or-later