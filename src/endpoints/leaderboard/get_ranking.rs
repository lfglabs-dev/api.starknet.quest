
/*
 this endpoint will return leaderboard ranking for one address
-> get user position
-> get leaderboard
1) iterate over one week timestamps and add total points
2) get full rankings for one week
3) split the total documents by page size and return the page which has the user
4) fetch previous and next page depending on request.
5) add flag value called "last index": to fetch the next set of documents from the position last index
5) "last index" can also act as a flag value to check if there is a previous page or next page
(last_index === -1 then no previous page && last_index === total.documents.length then no next page)
 */
