'use strict';

document.addEventListener('DOMContentLoaded', function() {
    var elements = document.querySelectorAll('.w3-input, .w3-checkbox, .w3-button');

    for (var i = 0; i < elements.length; i++) {
        elements[i].addEventListener('mouseover', function() {
            this.focus();
        });
        elements[i].addEventListener('mouseout', function() {
            this.blur();
        });
        elements[i].addEventListener('focus', function() {
            this.classList.add('focus');
        });
        elements[i].addEventListener('blur', function() {
            this.classList.remove('focus');
        });

    }

});