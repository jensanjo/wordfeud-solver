import heapq
import time
import pywordfeud

state = [
    "    t     c   f",
    "    e    he   o",
    "    r   bis g k",
    "    u  bol te v",
    "    gepof dimme",
    "      la vree e",
    "    qua   ene  ",
    "      Spoelen  ",
    "     s a   n   ",
    "     c d we    ",
    "     hadden    ",
    "    nu o   y   ",
    "  wrat siJzen  ",
    "    k     os   ",
    "   zerk   g    ",
]

board = pywordfeud.Board("NL", wordfile="wordlists/wordlist-nl.txt", state=state)
letters = "mdjenj*"
t0 = time.time()
top20 = board.calc_top_scores(letters, 20)
t1 = time.time()

print("{} scores in {:.2f} ms".format(len(top20), 1000*(t1-t0)))
for score in top20:
    print(score)

board.play_word("jAjem", 3, 5, False, True)
print(board)

