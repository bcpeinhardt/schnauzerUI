# Installation

**** *Heads up, these instructions are a bit technical. If you don't feel confident installing
this on your own, check out the youtube installation tutorial: todo create youtube install tutorial* ****

Note: SchnauzerUI executes against running chromedriver and geckodriver processes, but handles downloading,
running and stopping them itself. You should be able to do everything with just SchnauzerUI and the browser
you want to test in. Just make sure to have ports 4444 and 9515 open. If you don't know what that means,
then you're probably good to go.

To install SchnauzerUI, check [github](https://github.com/bcpeinhardt/schnauzerUI) for a 
download link or your operating system. Be sure to save it somewhere in your PATH.
Alternatively, if you have the rust toolchain installed, you can build it from source by running
`cargo install https://github.com/bcpeinhardt/schnauzerUI`
Also, go ahead and add `$HOME/.webdrivers` to your path as well. This is where chromedriver
and geckodriver will be downloaded to.

Hopefully, a browser opened up on your computer. In the terminal, you will be prompted for a name for your script.
Once you've done that, you'll be prompted for a command. Try `url "https://youtube.com"`.
The browser should navigate to youtube, and once the page has loaded, the terminal will ask whether you want to save
the step. Just hit enter to auto save and keep going. 

Congrats! You're all set up. To quit, type `exit` in the terminal and hit enter. It should close the browser for you,
but not before asking if you want to save your script. Type "N" to discard the script.
