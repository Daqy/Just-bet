# Ideas for improvements

- [ ] Add Mutex + Arc to click requests, ensure only one click gets handled at a time.
- [ ] move minesweeper game requests to db to middleware, so that im not duplicating the request everywhere.
  > [!IMPORTANT]
  > Minesweeper requests might not work with Mutex + Arc on click
