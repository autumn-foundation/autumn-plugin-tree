// The tree widget pack. The server sends a list of native radios or
// checkboxes in <details>. On `autumn:init` this file changes the list into
// a WAI-ARIA APG tree. The keyboard core in autumn-widgets.js moves the
// focus and opens and closes branches. This file does the rest: click,
// Enter, Space, Ctrl+A, the cascade, the tri-state and lazy loading.
//
// After the upgrade, each node is:
//
//   <li class="autumn-tree-node" role="none">
//     <input hidden>                       the form state
//     <div class="autumn-tree-row" role="treeitem" aria-owns="…-group">
//     <span class="autumn-tree-mixed">     the "partly selected" text
//     <ul class="autumn-tree-group" role="group">
//
// The row is the item, so its name and its visible text are the label.
(function () {
  'use strict';

  var ROOT = 'ul[data-autumn-tree]';
  var ITEM = '.autumn-tree-row[role="treeitem"]';
  // Items in a closed branch and loading rows are not reachable. The
  // keyboard core after autumn-web 0.10 also skips a hidden subtree.
  var REACHABLE = '[role="treeitem"]:not([role="group"][hidden] [role="treeitem"]):not(.autumn-tree-placeholder)';

  function closest(el, selector) {
    return el && el.closest ? el.closest(selector) : null;
  }
  function isMultiple(root) {
    return root.getAttribute('data-autumn-tree') === 'multiple';
  }
  function cascades(root) {
    return isMultiple(root) && root.getAttribute('data-autumn-tree-cascade') === 'descendants';
  }
  function child(el, selector) {
    for (var i = 0; el && i < el.children.length; i++) {
      if (el.children[i].matches(selector)) return el.children[i];
    }
    return null;
  }
  // The item, the input and the group are children of one <li>.
  function inputOf(item) {
    return child(item.parentElement, 'input');
  }
  function groupOf(item) {
    return child(item.parentElement, '.autumn-tree-group');
  }
  function itemOf(li) {
    return child(li, ITEM);
  }
  // The parent item, or null for a root item.
  function parentOf(item) {
    var group = item.parentElement.parentElement;
    return group && group.matches('.autumn-tree-group') ? itemOf(group.parentElement) : null;
  }
  // The enabled child items: they count for the state of a parent.
  function childItems(item) {
    var group = groupOf(item);
    if (!group) return [];
    return Array.prototype.map.call(group.children, itemOf).filter(function (it) {
      var input = it && inputOf(it);
      return input && !input.disabled;
    });
  }
  function itemsIn(el) {
    return Array.prototype.slice.call(el.querySelectorAll(ITEM));
  }
  function emit(el, name) {
    el.dispatchEvent(new CustomEvent(name, { bubbles: true, detail: { item: el } }));
  }

  // ── Upgrade ──────────────────────────────────────────────────────────

  // Change one server node to a tree item. Return the item, or null when
  // the node is already upgraded.
  function upgradeNode(li) {
    if (li.hasAttribute('role')) return null;
    var input = child(li, 'input');
    var details = child(li, 'details');
    var source = details ? child(details, 'summary') : child(li, 'label');
    var row = document.createElement('div');
    row.className = 'autumn-tree-row';
    ['autumn-tree-toggle', 'autumn-tree-mark'].forEach(function (name) {
      var span = document.createElement('span');
      span.className = name;
      span.setAttribute('aria-hidden', 'true');
      row.appendChild(span);
    });
    while (source && source.firstChild) row.appendChild(source.firstChild);
    li.insertBefore(row, details || source);
    // The "partly selected" text goes out of the row: it is a description.
    var mixed = row.querySelector('.autumn-tree-mixed');
    if (mixed) {
      mixed.setAttribute('aria-hidden', 'true');
      li.insertBefore(mixed, row.nextSibling);
    }
    if (details) {
      var group = child(details, 'ul');
      group.setAttribute('role', 'group');
      li.appendChild(group);
      row.setAttribute('aria-expanded', details.open ? 'true' : 'false');
      if (group.id) row.setAttribute('aria-owns', group.id);
      group.hidden = !details.open;
      details.remove();
    } else if (source) {
      source.remove();
    }
    // The row takes the id and the position of the node.
    if (li.id) {
      row.id = li.id;
      li.removeAttribute('id');
    }
    ['aria-level', 'aria-setsize', 'aria-posinset'].forEach(function (name) {
      if (li.hasAttribute(name)) {
        row.setAttribute(name, li.getAttribute(name));
        li.removeAttribute(name);
      }
    });
    var label = row.querySelector('.autumn-tree-label');
    if (label && label.id) row.setAttribute('aria-labelledby', label.id);
    li.setAttribute('role', 'none');
    row.setAttribute('role', 'treeitem');
    row.tabIndex = -1;
    if (input) {
      input.hidden = true;
      input.tabIndex = -1;
      // A hidden control cannot show the browser message. The tree has
      // aria-required, and the server checks the value.
      input.required = false;
      if (input.disabled) row.setAttribute('aria-disabled', 'true');
    }
    return row;
  }

  // A loading row: an item that is not reachable, with a status text.
  function upgradePlaceholder(li, root) {
    if (li.hasAttribute('role')) return;
    li.setAttribute('role', 'treeitem');
    li.setAttribute('aria-disabled', 'true');
    li.textContent = root.getAttribute('data-autumn-tree-loading') || '';
  }

  // Without htmx a lazy branch cannot load. Keep such a tree as the native
  // list: its links still show the children.
  function canUpgrade(root) {
    return !!window.htmx || !root.querySelector('[data-autumn-tree-src]');
  }

  // A polite status for screen readers: loading, and load errors.
  function statusOf(root) {
    var next = root.nextElementSibling;
    if (next && next.matches('.autumn-tree-status')) return next;
    var status = document.createElement('span');
    status.className = 'autumn-tree-status';
    status.setAttribute('role', 'status');
    root.parentNode.insertBefore(status, root.nextSibling);
    return status;
  }

  function upgrade(root) {
    if (!canUpgrade(root)) return;
    statusOf(root);
    // The framework marks the fieldset. The inputs are hidden, so the tree
    // carries the error state and the hint and error text.
    var fieldset = closest(root, 'fieldset');
    if (fieldset && fieldset.getAttribute('aria-invalid') === 'true') {
      root.setAttribute('aria-invalid', 'true');
    }
    if (fieldset && fieldset.hasAttribute('aria-describedby')) {
      root.setAttribute('aria-describedby', fieldset.getAttribute('aria-describedby'));
    }
    root.setAttribute('role', 'tree');
    root.classList.add('autumn-tree--enhanced');
    root.setAttribute('data-autumn-roving', REACHABLE);
    root.setAttribute('data-autumn-orientation', 'vertical');
    root.setAttribute('data-autumn-typeahead', '');
    if (isMultiple(root)) root.setAttribute('aria-multiselectable', 'true');
    if (root.getAttribute('data-autumn-tree-required') === 'true') {
      root.setAttribute('aria-required', 'true');
    }
    var fresh = [];
    root.querySelectorAll('li.autumn-tree-node').forEach(function (li) {
      var item = upgradeNode(li);
      if (item) fresh.push(item);
    });
    root.querySelectorAll('li.autumn-tree-placeholder').forEach(function (li) {
      upgradePlaceholder(li, root);
    });
    // A history snapshot can keep a load that did not end.
    root.querySelectorAll('[data-autumn-tree-src][aria-busy]:not([data-autumn-tree-loaded])')
      .forEach(function (group) { group.removeAttribute('aria-busy'); });
    // A checked parent checks its new children (lazy loading). Tree order:
    // a parent comes first.
    if (cascades(root)) {
      fresh.forEach(function (item) {
        var parent = parentOf(item);
        var input = inputOf(item);
        if (parent && inputOf(parent).checked && input && !input.disabled) input.checked = true;
      });
    }
    refresh(root);
    // A lazy node that the server opens loads at once.
    fresh.forEach(function (item) {
      if (item.getAttribute('aria-expanded') === 'true') load(item);
    });
  }

  // `autumn:init` fires on the page and on each htmx swap, before the core
  // sets the tab stop. The target can be a tree, or hold trees, or be in a
  // tree (lazy children).
  document.addEventListener('autumn:init', function (e) {
    var target = e.target;
    if (!target || !target.querySelectorAll) return;
    if (target.matches && target.matches(ROOT)) upgrade(target);
    target.querySelectorAll(ROOT).forEach(upgrade);
    var outer = closest(target.parentElement, ROOT);
    if (outer) upgrade(outer);
  });

  // ── State ────────────────────────────────────────────────────────────

  // Write the ARIA state from the inputs. A cascade parent with loaded
  // children takes its state from them.
  function refresh(root) {
    var items = itemsIn(root);
    if (!isMultiple(root)) {
      items.forEach(function (item) {
        var input = inputOf(item);
        if (input) item.setAttribute('aria-selected', input.checked ? 'true' : 'false');
      });
      return;
    }
    var derive = cascades(root);
    var state = new Map();
    for (var i = items.length - 1; i >= 0; i--) {
      var item = items[i];
      var input = inputOf(item);
      if (!input) continue;
      var value = input.checked ? 'true' : 'false';
      // A disabled node is frozen. A cut parent does not know all its
      // children.
      var partial = item.parentElement.hasAttribute('data-autumn-tree-partial');
      var kids = derive && !input.disabled && !partial ? childItems(item) : [];
      if (kids.length) {
        var on = kids.filter(function (k) { return state.get(k) === 'true'; }).length;
        var some = kids.some(function (k) { return state.get(k) !== 'false'; });
        value = on === kids.length ? 'true' : some ? 'mixed' : 'false';
        input.checked = value === 'true';
      }
      input.indeterminate = value === 'mixed';
      state.set(item, value);
      item.setAttribute('aria-checked', value);
      note(root, item, value === 'mixed');
    }
  }

  // The hidden "partly selected" text of a mixed parent.
  function note(root, item, mixed) {
    var li = item.parentElement;
    var span = child(li, '.autumn-tree-mixed');
    if (!span && mixed) {
      span = document.createElement('span');
      span.className = 'autumn-tree-mixed';
      span.setAttribute('aria-hidden', 'true');
      span.textContent = root.getAttribute('data-autumn-tree-mixed-text') || '';
      li.insertBefore(span, item.nextSibling);
    }
    if (!span) return;
    if (!span.id) span.id = item.id + '-mixed';
    span.hidden = !mixed;
    if (mixed) item.setAttribute('aria-describedby', span.id);
    else item.removeAttribute('aria-describedby');
  }

  // Select or check an item: click, Enter or Space.
  function activate(item) {
    var root = closest(item, ROOT);
    var input = inputOf(item);
    if (!root || !input || input.disabled) return;
    if (isMultiple(root)) {
      var on = item.getAttribute('aria-checked') !== 'true';
      input.checked = on;
      // The cascade stops at a disabled node.
      if (cascades(root)) {
        var stack = childItems(item);
        while (stack.length) {
          var next = stack.pop();
          inputOf(next).checked = on;
          stack = stack.concat(childItems(next));
        }
      }
    } else {
      if (input.checked) return;
      input.checked = true;
    }
    refresh(root);
    input.dispatchEvent(new Event('change', { bubbles: true }));
  }

  function checkAll(root) {
    var last = null;
    itemsIn(root).forEach(function (item) {
      var input = inputOf(item);
      if (input && !input.disabled && !input.checked) {
        input.checked = true;
        last = input;
      }
    });
    refresh(root);
    if (last) last.dispatchEvent(new Event('change', { bubbles: true }));
  }

  // Open or close a branch. The core does the same for Right and Left.
  function setOpen(item, open) {
    var group = groupOf(item);
    if (!group || item.getAttribute('aria-expanded') === String(open)) return;
    item.setAttribute('aria-expanded', open ? 'true' : 'false');
    group.hidden = !open;
    if (!open && group.contains(document.activeElement)) item.focus();
    emit(item, open ? 'autumn:expand' : 'autumn:collapse');
  }

  // ── Lazy children ────────────────────────────────────────────────────

  function load(item) {
    var group = groupOf(item);
    var root = closest(item, ROOT);
    if (!group || !root || !group.hasAttribute('data-autumn-tree-src')) return;
    if (group.hasAttribute('data-autumn-tree-loaded') || group.getAttribute('aria-busy') === 'true') return;
    var row = child(group, '.autumn-tree-placeholder');
    var loading = root.getAttribute('data-autumn-tree-loading') || '';
    if (row) row.textContent = loading;
    statusOf(root).textContent = loading;
    group.setAttribute('aria-busy', 'true');
    group.dispatchEvent(new CustomEvent('autumn-tree:load'));
  }

  document.addEventListener('autumn:expand', function (e) {
    var item = e.target;
    if (item && item.matches && item.matches(ITEM)) load(item);
  });

  // A group with a lazy URL, from an htmx event target.
  function lazyGroup(el) {
    return el && el.hasAttribute && el.hasAttribute('data-autumn-tree-src') ? el : null;
  }

  // The branch has children, or it is empty: an end node.
  function loaded(group) {
    var item = itemOf(group.parentElement);
    var root = closest(group, ROOT);
    group.removeAttribute('aria-busy');
    group.setAttribute('data-autumn-tree-loaded', '');
    if (item && !group.querySelector(ITEM)) {
      item.removeAttribute('aria-expanded');
      item.removeAttribute('aria-owns');
      group.remove();
    }
    if (root) {
      statusOf(root).textContent = '';
      refresh(root);
    }
  }

  function failed(group) {
    var root = closest(group, ROOT);
    group.removeAttribute('aria-busy');
    if (!root) return;
    var text = root.getAttribute('data-autumn-tree-failed') || '';
    var row = child(group, '.autumn-tree-placeholder');
    if (row) row.textContent = text;
    statusOf(root).textContent = text;
  }

  // htmx swapped the children in. The core and `autumn:init` ran first.
  document.addEventListener('htmx:afterSwap', function (e) {
    var group = lazyGroup(e.target);
    if (group) loaded(group);
  });

  // Each request ends here: a swap, no swap (204), an error, a timeout or
  // an abort.
  document.addEventListener('htmx:afterRequest', function (e) {
    var group = lazyGroup(e.target);
    if (!group || group.getAttribute('aria-busy') !== 'true') return;
    var xhr = e.detail && e.detail.xhr;
    if (e.detail && e.detail.successful && xhr && xhr.status === 204) loaded(group);
    else if (!(e.detail && e.detail.successful)) failed(group);
  });

  // htmx refuses the URL, for example another origin.
  document.addEventListener('htmx:invalidPath', function (e) {
    var group = lazyGroup(e.target);
    if (group) failed(group);
  });

  // ── Pointer and keys ─────────────────────────────────────────────────

  document.addEventListener('click', function (e) {
    var item = closest(e.target, '.autumn-tree--enhanced ' + ITEM);
    if (!item) return;
    // A disabled branch opens, but it does not select.
    if (closest(e.target, '.autumn-tree-toggle') && item.hasAttribute('aria-expanded')) {
      setOpen(item, item.getAttribute('aria-expanded') !== 'true');
      return;
    }
    if (item.getAttribute('aria-disabled') !== 'true') activate(item);
  });

  // Enter and Space select. Ctrl+A checks all. The core owns the arrows.
  document.addEventListener('keydown', function (e) {
    if (e.defaultPrevented || e.altKey || e.metaKey) return;
    var item = e.target;
    if (!item || !item.matches || !item.matches('.autumn-tree--enhanced ' + ITEM)) return;
    var root = closest(item, ROOT);
    if (e.ctrlKey && (e.key === 'a' || e.key === 'A') && isMultiple(root)) {
      e.preventDefault();
      checkAll(root);
      return;
    }
    if (e.ctrlKey || (e.key !== 'Enter' && e.key !== ' ')) return;
    e.preventDefault();
    if (item.getAttribute('aria-disabled') !== 'true') activate(item);
  });
})();
