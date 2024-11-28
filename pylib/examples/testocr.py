import heapq
import time
import pywordfeud_solver
import pywordfeud_ocr

#fn = "../wordfeud-ocr/lib/screenshots/screenshot_dutch.png"
fn = "../wordfeud-ocr/lib/screenshots/Screenshot_20210810-102249.png"
res = pywordfeud_ocr.recognize_screenshot_from_file(fn)
grid = res['board_ocr']
state = res['state_ocr']
letters = res['tray_ocr']

board = pywordfeud_solver.Board("NL", wordfile="wordlists/wordlist-nl.txt",
        state=state, grid=grid)

#t0 = time.time()
#top20 = board.calc_top_scores(letters, 20)
#t1 = time.time()

#print("{} scores in {:.2f} ms".format(len(top20), 1000*(t1-t0)))
#for score in top20:
#    print(score)

best_scores = board.find_best_scores(letters, 10)
for score in best_scores:
    print(score)

#board.play_word("jAjem", 3, 5, False, True)
#print(board)
#
