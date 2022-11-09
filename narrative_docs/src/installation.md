# Installation

**** *Heads up, these instructions are a bit technical. If you don't feel confident installing
this on your own, check out the youtube installation tutorial: todo create youtube install tutorial* ****

Schnauzer UI scripts are run against a selenium grid in standalone mode. You will need to download 
it from [the Selenium website](https://www.selenium.dev/downloads/). 

To run the grid, you'll need java installed on your system. Run the grid with
`java -jar <selenium-grid-file-name>.jar standalone`

Now in another terminal, we can cargo install Schnauzer UI. For this you'll
need cargo and rust on your system. In another temrinal, run
`cargo install https://github.com/bcpeinhardt/schnauzerUI`. 

Once it's done compiling, you should be all set up to start automating the 
Schnauzer UI way. To start up the repl, run
`schnauzerUI -i -p 4444 -b chrome`

The -i flage stands for "interactive", aka the repl.
The -p flag specifies the port your selenium grid is running on. It should have defaulted to 4444, so that's what we'll tell SchnauzerUI.
The -b flage specifies the browser you want to use. Supports "chrome" and "firefox".

Hopefully, a browser opened up on your computer. In the terminal, you will be prompted for a name for your script.
Once you've done that, you'll be prompted for a command. Try `url "https://youtube.com"`.
The browser should navigate to youtube, and once the page has loaded, the terminal will ask whether you want to save
the step. Just hit enter to auto save and keep going. 

Congrats! You're all set up. To quit, type `exit` in the terminal and hit enter. It should close the browser for you,
but not before asking if you want to save your script. Type "N" to discard the script.
