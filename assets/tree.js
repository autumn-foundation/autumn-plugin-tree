// The tree widget pack. The server sends a list of native radios or
// checkboxes in <details>. On `autumn:init` this file changes the list into
// a WAI-ARIA APG tree. The keyboard core in autumn-widgets.js moves the
// focus and opens and closes branches. This file does the rest: click,
// Enter, Space, Ctrl+A, the cascade, the tri-state and lazy loading.
(function () {
  'use strict';

  var ROOT = 'ul[data-autumn-tree]';
  var ITEM = '[role="treeitem"]';
  // Items in a closed branch and loading rows are not reachable.
  var REACHABLE = '[role="treeitem"]:not([role="treeitem"][aria-expanded="false"] [role="treeitem"]):not(.autumn-tree-placeholder)';

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
    for (var i = 0; i < el.children.length; i++) {
      if (el.children[i].matches(selector)) return el.children[i];
    }
    return null;
  }
  function inputOf(item) {
    return child(item, 'input');
  }
  function groupOf(item) {
    return child(item, '[role="group"]');
  }
  function rowOf(item) {
    return child(item, '.autumn-tree-row');
  }
  // The child items that count for the state of a parent.
  function childItems(item) {
    var group = groupOf(item);
    if (!group) return [];
    return Array.prototype.filter.call(group.children, function (li) {
      var input = li.matches(ITEM) && inputOf(li);
      return input && !input.disabled;
    });
  }
  function emit(el, name) {
    el.dispatchEvent(new CustomEvent(name, { bubbles: true, detail: { item: el } }));
  }

  // ── Upgrade ──────────────────────────────────────────────────────────

  function upgradeItem(li) {
    if (li.hasAttribute('role')) return;
    var input = inputOf(li);
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
    if (details) {
      var group = child(details, 'ul');
      group.setAttribute('role', 'group');
      li.appendChild(group);
      li.setAttribute('aria-expanded', details.open ? 'true' : 'false');
      group.hidden = !details.open;
      details.remove();
    } else if (source) {
      source.remove();
    }
    var label = row.querySelector('.autumn-tree-label');
    if (label && label.id) li.setAttribute('aria-labelledby', label.id);
    li.setAttribute('role', 'treeitem');
    li.tabIndex = -1;
    if (input) {
      input.hidden = true;
      input.tabIndex = -1;
      // A hidden control cannot show the browser message. The tree has
      // aria-required, and the server checks the value.
      input.required = false;
      if (input.disabled) li.setAttribute('aria-disabled', 'true');
    }
  }

  // A loading row: an item that is not reachable, with a status text.
  function upgradePlaceholder(li, root) {
    if (li.hasAttribute('role')) return;
    li.setAttribute('role', 'treeitem');
    li.setAttribute('aria-disabled', 'true');
    li.textContent = root.getAttribute('data-autumn-tree-loading') || '';
  }

  function upgrade(root) {
    root.setAttribute('role', 'tree');
    root.classList.add('autumn-tree--enhanced');
    root.setAttribute('data-autumn-roving', REACHABLE);
    root.setAttribute('data-autumn-orientation', 'vertical');
    root.setAttribute('data-autumn-typeahead', '');
    if (isMultiple(root)) root.setAttribute('aria-multiselectable', 'true');
    if (root.getAttribute('data-autumn-tree-required') === 'true') {
      root.setAttribute('aria-required', 'true');
    }
    root.querySelectorAll('li.autumn-tree-node').forEach(upgradeItem);
    root.querySelectorAll('li.autumn-tree-placeholder').forEach(function (li) {
      upgradePlaceholder(li, root);
    });
    refresh(root);
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
    var items = root.querySelectorAll('li.autumn-tree-node' + ITEM);
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
      var kids = derive && !input.disabled ? childItems(item) : [];
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
    var row = rowOf(item);
    var span = row && row.querySelector('.autumn-tree-mixed');
    if (!span && mixed && row) {
      span = document.createElement('span');
      span.className = 'autumn-tree-mixed';
      span.textContent = root.getAttribute('data-autumn-tree-mixed-text') || '';
      row.appendChild(span);
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
      if (cascades(root)) {
        var group = groupOf(item);
        if (group) group.querySelectorAll('li.autumn-tree-node' + ITEM).forEach(function (d) {
          var di = inputOf(d);
          if (di && !di.disabled) di.checked = on;
        });
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
    root.querySelectorAll('li.autumn-tree-node' + ITEM).forEach(function (item) {
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
    if (!window.htmx) {
      if (row) row.textContent = root.getAttribute('data-autumn-tree-failed') || '';
      return;
    }
    if (row) row.textContent = root.getAttribute('data-autumn-tree-loading') || '';
    group.setAttribute('aria-busy', 'true');
    group.dispatchEvent(new CustomEvent('autumn-tree:load'));
  }

  document.addEventListener('autumn:expand', function (e) {
    var item = e.target;
    if (item && item.matches && item.matches('li.autumn-tree-node' + ITEM)) load(item);
  });

  // htmx swapped the children in. The core and `autumn:init` ran first.
  document.addEventListener('htmx:afterSwap', function (e) {
    var group = e.target;
    if (!group || !group.hasAttribute || !group.hasAttribute('data-autumn-tree-src')) return;
    var item = group.parentElement;
    var root = closest(group, ROOT);
    group.removeAttribute('aria-busy');
    group.setAttribute('data-autumn-tree-loaded', '');
    var input = inputOf(item);
    if (root && cascades(root) && input && input.checked) {
      group.querySelectorAll('li.autumn-tree-node' + ITEM).forEach(function (d) {
        var di = inputOf(d);
        if (di && !di.disabled) di.checked = true;
      });
    }
    // An empty branch is an end node.
    if (!group.querySelector(ITEM)) {
      item.removeAttribute('aria-expanded');
      group.remove();
    }
    if (root) refresh(root);
  });

  function failed(e) {
    var group = e.target;
    if (!group || !group.hasAttribute || !group.hasAttribute('data-autumn-tree-src')) return;
    var root = closest(group, ROOT);
    group.removeAttribute('aria-busy');
    var row = child(group, '.autumn-tree-placeholder');
    if (row && root) row.textContent = root.getAttribute('data-autumn-tree-failed') || '';
  }
  document.addEventListener('htmx:responseError', failed);
  document.addEventListener('htmx:sendError', failed);

  // ── Pointer and keys ─────────────────────────────────────────────────

  document.addEventListener('click', function (e) {
    var row = closest(e.target, '.autumn-tree--enhanced .autumn-tree-row');
    var item = row && row.parentElement;
    if (!item || !item.matches(ITEM) || item.getAttribute('aria-disabled') === 'true') return;
    if (closest(e.target, '.autumn-tree-toggle') && item.hasAttribute('aria-expanded')) {
      setOpen(item, item.getAttribute('aria-expanded') !== 'true');
      return;
    }
    activate(item);
  });

  // Enter and Space select. Ctrl+A checks all. The core owns the arrows.
  document.addEventListener('keydown', function (e) {
    if (e.defaultPrevented || e.altKey || e.metaKey) return;
    var item = e.target;
    if (!item || !item.matches || !item.matches('.autumn-tree--enhanced ' + ITEM)) return;
    var root = closest(item, ROOT);
    if ((e.ctrlKey && (e.key === 'a' || e.key === 'A')) && isMultiple(root)) {
      e.preventDefault();
      checkAll(root);
      return;
    }
    if (e.ctrlKey || (e.key !== 'Enter' && e.key !== ' ')) return;
    e.preventDefault();
    if (item.getAttribute('aria-disabled') !== 'true') activate(item);
  });
})();
