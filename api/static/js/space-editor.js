/*
 *   This content is licensed according to the W3C Software License at
 *   https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
 *
 *   File:   menubar-editor.js
 *
 *   Desc:   Creates a menubar to control the styling of text in a textarea element
 */

/* global StyleManager */

'use strict';

class MenubarEditor {
    constructor(domNode) {
        this.domNode = domNode;
        this.menubarNode = domNode.querySelector('[role=menubar]');
        this.textareaNode = domNode.querySelector('.post-content');
        this.actionManager = new StyleManager(this.textareaNode);

        this.popups = [];
        this.menuitemGroups = {};
        this.menuOrientation = {};
        this.isPopup = {};

        this.firstChars = {}; // see Menubar init method
        this.firstMenuitem = {}; // see Menubar init method
        this.lastMenuitem = {}; // see Menubar init method

        this.initMenu(this.menubarNode);
        this.domNode.addEventListener('focusin', this.onFocusin.bind(this));
        this.domNode.addEventListener('focusout', this.onFocusout.bind(this));

        window.addEventListener(
            'pointerdown',
            this.onBackgroundPointerdown.bind(this),
            true
        );
    }

    getMenuitems(domNode) {
        var nodes = [];

        var initMenu = this.initMenu.bind(this);
        var getGroupId = this.getGroupId.bind(this);
        var menuitemGroups = this.menuitemGroups;
        var popups = this.popups;

        function findMenuitems(node, group) {
            var role, flag, groupId;

            while (node) {
                flag = true;
                role = node.getAttribute('role');

                switch (role) {
                    case 'menu':
                        node.tabIndex = -1;
                        initMenu(node);
                        flag = false;
                        break;

                    case 'group':
                        groupId = getGroupId(node);
                        menuitemGroups[groupId] = [];
                        break;

                    case 'menuitem':
                    case 'menuitemradio':
                    case 'menuitemcheckbox':
                        if (node.getAttribute('aria-haspopup') === 'true') {
                            popups.push(node);
                        }
                        nodes.push(node);
                        if (group) {
                            group.push(node);
                        }
                        break;

                    default:
                        break;
                }

                if (flag && node.firstElementChild) {
                    findMenuitems(node.firstElementChild, menuitemGroups[groupId]);
                }

                node = node.nextElementSibling;
            }
        }

        findMenuitems(domNode.firstElementChild, false);

        return nodes;
    }

    initMenu(menu) {
        var i, menuitems, menuitem, role;

        var menuId = this.getMenuId(menu);

        menuitems = this.getMenuitems(menu);
        this.menuOrientation[menuId] = this.getMenuOrientation(menu);
        this.isPopup[menuId] = menu.getAttribute('role') === 'menu';

        this.menuitemGroups[menuId] = [];
        this.firstChars[menuId] = [];
        this.firstMenuitem[menuId] = null;
        this.lastMenuitem[menuId] = null;

        for (i = 0; i < menuitems.length; i++) {
            menuitem = menuitems[i];
            role = menuitem.getAttribute('role');

            if (role.indexOf('menuitem') < 0) {
                continue;
            }

            menuitem.tabIndex = -1;
            this.menuitemGroups[menuId].push(menuitem);
            this.firstChars[menuId].push(menuitem.textContent[0].toLowerCase());

            menuitem.addEventListener('keydown', this.onKeydown.bind(this));
            menuitem.addEventListener('click', this.onMenuitemClick.bind(this));

            menuitem.addEventListener(
                'pointerover',
                this.onMenuitemPointerover.bind(this)
            );

            if (!this.firstMenuitem[menuId]) {
                if (this.hasPopup(menuitem)) {
                    menuitem.tabIndex = 0;
                }
                this.firstMenuitem[menuId] = menuitem;
            }
            this.lastMenuitem[menuId] = menuitem;
        }
    }

    /* MenubarEditor FOCUS MANAGEMENT METHODS */

    setFocusToMenuitem(menuId, newMenuitem) {
        var isAnyPopupOpen = this.isAnyPopupOpen();

        this.closePopupAll(newMenuitem);

        if (this.hasPopup(newMenuitem)) {
            if (isAnyPopupOpen) {
                this.openPopup(newMenuitem);
            }
        } else {
            var menu = this.getMenu(newMenuitem);
            var cmi = menu.previousElementSibling;
            if (!this.isOpen(cmi)) {
                this.openPopup(cmi);
            }
        }

        if (this.hasPopup(newMenuitem)) {
            if (this.menuitemGroups[menuId]) {
                this.menuitemGroups[menuId].forEach(function (item) {
                    item.tabIndex = -1;
                });
            }
            newMenuitem.tabIndex = 0;
        }

        newMenuitem.focus();
    }

    setFocusToFirstMenuitem(menuId) {
        this.setFocusToMenuitem(menuId, this.firstMenuitem[menuId]);
    }

    setFocusToLastMenuitem(menuId) {
        this.setFocusToMenuitem(menuId, this.lastMenuitem[menuId]);
    }

    setFocusToPreviousMenuitem(menuId, currentMenuitem) {
        var newMenuitem, index;

        if (currentMenuitem === this.firstMenuitem[menuId]) {
            newMenuitem = this.lastMenuitem[menuId];
        } else {
            index = this.menuitemGroups[menuId].indexOf(currentMenuitem);
            newMenuitem = this.menuitemGroups[menuId][index - 1];
        }

        this.setFocusToMenuitem(menuId, newMenuitem);

        return newMenuitem;
    }

    setFocusToNextMenuitem(menuId, currentMenuitem) {
        var newMenuitem, index;

        if (currentMenuitem === this.lastMenuitem[menuId]) {
            newMenuitem = this.firstMenuitem[menuId];
        } else {
            index = this.menuitemGroups[menuId].indexOf(currentMenuitem);
            newMenuitem = this.menuitemGroups[menuId][index + 1];
        }
        this.setFocusToMenuitem(menuId, newMenuitem);

        return newMenuitem;
    }

    setFocusByFirstCharacter(menuId, currentMenuitem, char) {
        var start, index;

        char = char.toLowerCase();

        // Get start index for search based on position of currentItem
        start = this.menuitemGroups[menuId].indexOf(currentMenuitem) + 1;
        if (start >= this.menuitemGroups[menuId].length) {
            start = 0;
        }

        // Check remaining slots in the menu
        index = this.getIndexFirstChars(menuId, start, char);

        // If not found in remaining slots, check from beginning
        if (index === -1) {
            index = this.getIndexFirstChars(menuId, 0, char);
        }

        // If match was found...
        if (index > -1) {
            this.setFocusToMenuitem(menuId, this.menuitemGroups[menuId][index]);
        }
    }

    // Utilities

    getIndexFirstChars(menuId, startIndex, char) {
        for (var i = startIndex; i < this.firstChars[menuId].length; i++) {
            if (char === this.firstChars[menuId][i]) {
                return i;
            }
        }
        return -1;
    }

    isPrintableCharacter(str) {
        return str.length === 1 && str.match(/\S/);
    }

    getIdFromAriaLabel(node) {
        var id = node.getAttribute('aria-label');
        if (id) {
            id = id.trim().toLowerCase().replace(' ', '-').replace('/', '-');
        }
        return id;
    }

    getMenuOrientation(node) {
        var orientation = node.getAttribute('aria-orientation');

        if (!orientation) {
            var role = node.getAttribute('role');

            switch (role) {
                case 'menubar':
                    orientation = 'horizontal';
                    break;

                case 'menu':
                    orientation = 'vertical';
                    break;

                default:
                    break;
            }
        }

        return orientation;
    }

    getDataOption(node) {
        var option = false;
        var hasOption = node.hasAttribute('data-option');
        var role = node.hasAttribute('role');

        if (!hasOption) {
            while (node && !hasOption && role !== 'menu' && role !== 'menubar') {
                node = node.parentNode;
                if (node) {
                    role = node.getAttribute('role');
                    hasOption = node.hasAttribute('data-option');
                }
            }
        }

        if (node) {
            option = node.getAttribute('data-option');
        }

        return option;
    }

    getGroupId(node) {
        var id = false;
        var role = node.getAttribute('role');

        while (node && role !== 'group' && role !== 'menu' && role !== 'menubar') {
            node = node.parentNode;
            if (node) {
                role = node.getAttribute('role');
            }
        }

        if (node) {
            id = role + '-' + this.getIdFromAriaLabel(node);
        }

        return id;
    }

    getMenuId(node) {
        var id = false;
        var role = node.getAttribute('role');

        while (node && role !== 'menu' && role !== 'menubar') {
            node = node.parentNode;
            if (node) {
                role = node.getAttribute('role');
            }
        }

        if (node) {
            id = role + '-' + this.getIdFromAriaLabel(node);
        }

        return id;
    }

    getMenu(menuitem) {
        var menu = menuitem;
        var role = menuitem.getAttribute('role');

        while (menu && role !== 'menu' && role !== 'menubar') {
            menu = menu.parentNode;
            if (menu) {
                role = menu.getAttribute('role');
            }
        }

        return menu;
    }

    toggleCheckbox(menuitem) {
        if (menuitem.getAttribute('aria-checked') === 'true') {
            menuitem.setAttribute('aria-checked', 'false');
            return false;
        }
        menuitem.setAttribute('aria-checked', 'true');
        return true;
    }

    setRadioButton(menuitem) {
        var groupId = this.getGroupId(menuitem);
        var radiogroupItems = this.menuitemGroups[groupId];
        radiogroupItems.forEach(function (item) {
            item.setAttribute('aria-checked', 'false');
        });
        menuitem.setAttribute('aria-checked', 'true');
        return menuitem.textContent;
    }

    updateFontSizeMenu(menuId) {
        var fontSizeMenuitems = this.menuitemGroups[menuId];
        var currentValue = this.actionManager.getFontSize();

        for (var i = 0; i < fontSizeMenuitems.length; i++) {
            var mi = fontSizeMenuitems[i];
            var dataOption = mi.getAttribute('data-option');
            var value = mi.textContent.trim().toLowerCase();

            switch (dataOption) {
                case 'font-smaller':
                    if (currentValue === 'x-small') {
                        mi.setAttribute('aria-disabled', 'true');
                    } else {
                        mi.removeAttribute('aria-disabled');
                    }
                    break;

                case 'font-larger':
                    if (currentValue === 'x-large') {
                        mi.setAttribute('aria-disabled', 'true');
                    } else {
                        mi.removeAttribute('aria-disabled');
                    }
                    break;

                default:
                    if (currentValue === value) {
                        mi.setAttribute('aria-checked', 'true');
                    } else {
                        mi.setAttribute('aria-checked', 'false');
                    }
                    break;
            }
        }
    }

    // Popup menu methods

    isAnyPopupOpen() {
        for (var i = 0; i < this.popups.length; i++) {
            if (this.popups[i].getAttribute('aria-expanded') === 'true') {
                return true;
            }
        }
        return false;
    }

    openPopup(menuitem) {
        // set aria-expanded attribute
        var popupMenu = menuitem.nextElementSibling;

        var rect = menuitem.getBoundingClientRect();

        // set CSS properties
        popupMenu.style.position = 'absolute';
        popupMenu.style.top = rect.height - 3 + 'px';
        popupMenu.style.left = '0px';
        popupMenu.style.zIndex = 100;
        popupMenu.style.display = 'block';

        menuitem.setAttribute('aria-expanded', 'true');

        return this.getMenuId(popupMenu);
    }

    closePopup(menuitem) {
        var menu, cmi;

        if (this.hasPopup(menuitem)) {
            if (this.isOpen(menuitem)) {
                menuitem.setAttribute('aria-expanded', 'false');
                menuitem.nextElementSibling.style.display = 'none';
                menuitem.nextElementSibling.style.zIndex = 0;
            }
        } else {
            menu = this.getMenu(menuitem);
            cmi = menu.previousElementSibling;
            cmi.setAttribute('aria-expanded', 'false');
            cmi.focus();
            menu.style.display = 'none';
            menu.style.zIndex = 0;
        }
        return cmi;
    }

    doesNotContain(popup, menuitem) {
        if (menuitem) {
            return !popup.nextElementSibling.contains(menuitem);
        }
        return true;
    }

    closePopupAll(menuitem) {
        if (typeof menuitem !== 'object') {
            menuitem = false;
        }

        for (var i = 0; i < this.popups.length; i++) {
            var popup = this.popups[i];
            if (this.isOpen(popup) && this.doesNotContain(popup, menuitem)) {
                this.closePopup(popup);
            }
        }
    }

    hasPopup(menuitem) {
        return menuitem.getAttribute('aria-haspopup') === 'true';
    }

    isOpen(menuitem) {
        return menuitem.getAttribute('aria-expanded') === 'true';
    }

    // Menu event handlers

    onFocusin() {
        this.domNode.classList.add('focus');
    }

    onFocusout() {
        this.domNode.classList.remove('focus');
    }

    onBackgroundPointerdown(event) {
        if (!this.menubarNode.contains(event.target)) {
            this.closePopupAll();
        }
    }

    onKeydown(event) {
        var tgt = event.currentTarget,
            key = event.key,
            flag = false,
            menuId = this.getMenuId(tgt),
            id,
            popupMenuId,
            mi,
            role,
            option,
            value;

        switch (key) {
            case ' ':
            case 'Enter':
                if (this.hasPopup(tgt)) {
                    popupMenuId = this.openPopup(tgt);
                    this.setFocusToFirstMenuitem(popupMenuId);
                } else {
                    role = tgt.getAttribute('role');
                    option = this.getDataOption(tgt);
                    switch (role) {
                        case 'menuitem':
                            this.actionManager.setOption(option, tgt.textContent);
                            break;

                        case 'menuitemcheckbox':
                            value = this.toggleCheckbox(tgt);
                            this.actionManager.setOption(option, value);
                            break;

                        case 'menuitemradio':
                            value = this.setRadioButton(tgt);
                            this.actionManager.setOption(option, value);
                            break;

                        default:
                            break;
                    }

                    if (this.getMenuId(tgt) === 'menu-size') {
                        this.updateFontSizeMenu('menu-size');
                    }
                    this.closePopup(tgt);
                }
                flag = true;
                break;

            case 'ArrowDown':
            case 'Down':
                if (this.menuOrientation[menuId] === 'vertical') {
                    this.setFocusToNextMenuitem(menuId, tgt);
                    flag = true;
                } else {
                    if (this.hasPopup(tgt)) {
                        popupMenuId = this.openPopup(tgt);
                        this.setFocusToFirstMenuitem(popupMenuId);
                        flag = true;
                    }
                }
                break;

            case 'Esc':
            case 'Escape':
                this.closePopup(tgt);
                flag = true;
                break;

            case 'Left':
            case 'ArrowLeft':
                if (this.menuOrientation[menuId] === 'horizontal') {
                    this.setFocusToPreviousMenuitem(menuId, tgt);
                    flag = true;
                } else {
                    mi = this.closePopup(tgt);
                    id = this.getMenuId(mi);
                    mi = this.setFocusToPreviousMenuitem(id, mi);
                    this.openPopup(mi);
                }
                break;

            case 'Right':
            case 'ArrowRight':
                if (this.menuOrientation[menuId] === 'horizontal') {
                    this.setFocusToNextMenuitem(menuId, tgt);
                    flag = true;
                } else {
                    mi = this.closePopup(tgt);
                    id = this.getMenuId(mi);
                    mi = this.setFocusToNextMenuitem(id, mi);
                    this.openPopup(mi);
                }
                break;

            case 'Up':
            case 'ArrowUp':
                if (this.menuOrientation[menuId] === 'vertical') {
                    this.setFocusToPreviousMenuitem(menuId, tgt);
                    flag = true;
                } else {
                    if (this.hasPopup(tgt)) {
                        popupMenuId = this.openPopup(tgt);
                        this.setFocusToLastMenuitem(popupMenuId);
                        flag = true;
                    }
                }
                break;

            case 'Home':
            case 'PageUp':
                this.setFocusToFirstMenuitem(menuId, tgt);
                flag = true;
                break;

            case 'End':
            case 'PageDown':
                this.setFocusToLastMenuitem(menuId, tgt);
                flag = true;
                break;

            case 'Tab':
                this.closePopup(tgt);
                break;

            default:
                if (this.isPrintableCharacter(key)) {
                    this.setFocusByFirstCharacter(menuId, tgt, key);
                    flag = true;
                }
                break;
        }

        if (flag) {
            event.stopPropagation();
            event.preventDefault();
        }
    }

    onMenuitemClick(event) {
        var tgt = event.currentTarget;
        var value;

        if (this.hasPopup(tgt)) {
            if (this.isOpen(tgt)) {
                this.closePopup(tgt);
            } else {
                var menuId = this.openPopup(tgt);
                this.setFocusToMenuitem(menuId, tgt);
            }
        } else {
            var role = tgt.getAttribute('role');
            var option = this.getDataOption(tgt);
            switch (role) {
                case 'menuitem':
                    this.actionManager.setOption(option, tgt.textContent);
                    break;

                case 'menuitemcheckbox':
                    value = this.toggleCheckbox(tgt);
                    this.actionManager.setOption(option, value);
                    break;

                case 'menuitemradio':
                    value = this.setRadioButton(tgt);
                    this.actionManager.setOption(option, value);
                    break;

                default:
                    break;
            }

            if (this.getMenuId(tgt) === 'menu-size') {
                this.updateFontSizeMenu('menu-size');
            }
            this.closePopup(tgt);
        }

        event.stopPropagation();
        event.preventDefault();
    }

    onMenuitemPointerover(event) {
        var tgt = event.currentTarget;

        if (this.isAnyPopupOpen() && this.getMenu(tgt)) {
            this.setFocusToMenuitem(this.getMenu(tgt), tgt);
        }
    }
}

// Initialize menubar editor

window.addEventListener('load', function () {
    var menubarEditors = document.querySelectorAll('.menubar-editor');
    for (var i = 0; i < menubarEditors.length; i++) {
        new MenubarEditor(menubarEditors[i]);
    }
});
/*
 *   This content is licensed according to the W3C Software License at
 *   https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
 *
 *   File:   TextStyling.js
 *
 *   Desc:   Styling functions for changing the style of an item
 */

'use strict';

/* exported StyleManager */

class StyleManager {
    constructor(node) {
        this.node = node;
        this.fontSize = '15px';
    }

    setFontFamily(value) {
        this.node.style.fontFamily = value;
    }

    setTextDecoration(value) {
        this.node.style.textDecoration = value;
    }

    setTextAlign(value) {
        this.node.style.textAlign = value;
    }

    setFontSize(value) {
        this.fontSize = value;
        this.node.style.fontSize = value;
    }

    setColor(value) {
        this.node.style.color = value;
    }

    setBold(flag) {
        if (flag) {
            this.node.style.fontWeight = 'bold';
        } else {
            this.node.style.fontWeight = 'normal';
        }
    }

    setItalic(flag) {
        if (flag) {
            this.node.style.fontStyle = 'italic';
        } else {
            this.node.style.fontStyle = 'normal';
        }
    }

    fontSmaller() {
        switch (this.fontSize) {
            case 'small':
                this.setFontSize('x-small');
                break;

            case '15px':
                this.setFontSize('small');
                break;

            case 'large':
                this.setFontSize('15px');
                break;

            case 'x-large':
                this.setFontSize('large');
                break;

            default:
                break;
        } // end switch
    }

    fontLarger() {
        switch (this.fontSize) {
            case 'x-small':
                this.setFontSize('small');
                break;

            case 'small':
                this.setFontSize('15px');
                break;

            case '15px':
                this.setFontSize('large');
                break;

            case 'large':
                this.setFontSize('x-large');
                break;

            default:
                break;
        } // end switch
    }

    isMinFontSize() {
        return this.fontSize === 'x-small';
    }

    isMaxFontSize() {
        return this.fontSize === 'x-large';
    }

    getFontSize() {
        return this.fontSize;
    }

    setOption(option, value) {
        option = option.toLowerCase();
        if (typeof value === 'string') {
            value = value.toLowerCase();
        }

        switch (option) {
            case 'font-bold':
                this.setBold(value);
                break;

            case 'font-color':
                this.setColor(value);
                break;

            case 'font-family':
                this.setFontFamily(value);
                break;

            case 'font-smaller':
                this.fontSmaller();
                break;

            case 'font-larger':
                this.fontLarger();
                break;

            case 'font-size':
                this.setFontSize(value);
                break;

            case 'font-italic':
                this.setItalic(value);
                break;

            case 'text-align':
                this.setTextAlign(value);
                break;

            case 'text-decoration':
                this.setTextDecoration(value);
                break;

            default:
                break;
        } // end switch
    }
}
