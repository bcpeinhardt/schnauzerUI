# What Schnauzer UI knows

Schnauzer UI tries to be smart about the way it interacts with HTML, so the person 
writing the test rarely has to look the underlying HTML of a page. This page outlines
some of the strategies it takes to achieve that.

## Smart Swap

Schnauzer UI tries to make it easy to locate and interact with web elements based 
on only visible information on the page (i.e. without having to look at HTML).
One way it does this is by acknowledging that HTML elements are often used together
in common patterns (many are designed this way).

When Schnauzer UI can, it will try to make a judgement call about the element you
actually want to interact with vs the element you were able to locate on the page, and 
swap them out for appropriate commands. 

For input, textarea, and select elements, if you are able to locate 

 - A span or label containing the element
 - A span or label directly preceding the element
 - A span or label roughly near the element

then Schnauzer UI will swap the element on `click` and `type` commands

Additionally, `select` commands will do the same, as well as check to see if the currently located element is 
an option inside a select.

## Clicking and Typing

Often times, the real world components we're testing are *fancy*.
The produced HTML can have several layers that shift around while we interact with them.
Schnauzer UI tries to make these components easier to test by acting as closely to a person as possible.

The `click` command does not click the located element. It registers a click event at the center of the elements
location on the page. This helps to reduce headaches around which element should actually be clicked
(in my experience, dealing with click intercepted errors is half of day to day selenium use).

Similarly, the `type` command doesn't type into the located element. It clicks the currently located element, and
then types into the **active element**. This is what people do, and it helps with fancy search dropdowns and things
that use javascript to swap out elements underneath you after registering a click.

## Under

HTML is fundamentally a tree structure, and it's structure isn't necessarily related to the way a webpage is
layed out (CSS and JavaScript have a tendency to move things around).
Often though, HTML elements are composed something like this.

```html
<div id="container">
    <div id="header">
        I am a heading of some section of this page
    </div>
    <div id="content">
        <p>I am some content</p>
        <div id="sub-content">
            ... more html
        </div>
    </div>
</div>
```

Pieces of HTML that related to each other get wrapped up in a container, with some kind of title/heading
at the top followed by the content of the component.
Often the HTML is more deeply nested than this. For example, here's the sample HTML for a Bootstrap card

```html
<div class="card" style="width: 18rem;">
  <img class="card-img-top" src="..." alt="Card image cap">
  <div class="card-body">
    <h5 class="card-title">Card title</h5>
    <p class="card-text">Some quick example text to build on the card title and make up the bulk of the card's content.</p>
  </div>
  <ul class="list-group list-group-flush">
    <li class="list-group-item">Cras justo odio</li>
    <li class="list-group-item">Dapibus ac facilisis in</li>
    <li class="list-group-item">Vestibulum at eros</li>
  </ul>
  <div class="card-body">
    <a href="#" class="card-link">Card link</a>
    <a href="#" class="card-link">Another link</a>
  </div>
</div>
```

Schnauzer UI takes advantage of this common organization strategy when helping you locate elements with visual
information. The `under` command changes the way locators work, so that the search for your element begins where
you want it to. 

One could type `under "Card title" locate "Card link" and click`, and expect the Card link to be correctly selected and clicked. Schnauzer UI will start with the h5 element and walk up the tree to the parent element, until the current element contains the one it's looking for.
This is especially useful for buttons/labels/placeholders that get repeated several places on a page. Think "add to cart", "see more...", etc. on gallery type pages.


