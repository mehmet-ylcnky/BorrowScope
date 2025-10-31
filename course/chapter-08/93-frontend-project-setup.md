# Section 93: Frontend Project Setup

## Learning Objectives

By the end of this section, you will:
- Master frontend project setup implementation techniques
- Understand best practices and patterns
- Build production-ready components
- Handle edge cases and errors
- Optimize for performance
- Write comprehensive tests

## Prerequisites

- Completed previous sections in Chapter 8
- Understanding of modern JavaScript/TypeScript
- Familiarity with web development concepts
- Knowledge of component architecture
- Basic understanding of visualization libraries

## Introduction

This section provides comprehensive coverage of frontend project setup in the BorrowScope application. We'll explore implementation details, best practices, common pitfalls, and optimization techniques to build a production-ready feature.

Frontend Project Setup is a critical component of the BorrowScope UI, enabling users to effectively visualize and interact with Rust ownership graphs. This section covers everything from basic setup to advanced features and performance optimization.

## Core Concepts

### Architecture Overview

The frontend project setup system follows these key principles:

1. **Modularity**: Components are self-contained and reusable
2. **Performance**: Optimized for large datasets
3. **Maintainability**: Clear separation of concerns
4. **Extensibility**: Easy to add new features
5. **Testability**: Comprehensive test coverage

### Design Patterns

We employ several design patterns:

- **Component Pattern**: Encapsulated UI components
- **Observer Pattern**: Event-driven updates
- **Strategy Pattern**: Pluggable algorithms
- **Factory Pattern**: Object creation
- **Singleton Pattern**: Shared state management

## Implementation

### Main Component

```javascript
/**
 * Main component for Frontend Project Setup
 * Handles initialization, rendering, and lifecycle management
 */
export class MainComponent {
    constructor(options = {}) {
        this.options = {
            container: null,
            data: null,
            config: {},
            ...options
        };
        
        this.state = {
            initialized: false,
            loading: false,
            error: null,
            data: null
        };
        
        this.handlers = new Map();
        this.subscriptions = [];
        this.cache = new Map();
    }
    
    /**
     * Initialize the component
     */
    async initialize() {
        if (this.state.initialized) {
            console.warn('Component already initialized');
            return;
        }
        
        try {
            this.state.loading = true;
            
            // Validate options
            this.validateOptions();
            
            // Setup DOM
            await this.setupDOM();
            
            // Load resources
            await this.loadResources();
            
            // Attach event listeners
            this.attachEventListeners();
            
            // Initial render
            this.render();
            
            this.state.initialized = true;
            this.state.loading = false;
            
            this.emit('initialized');
        } catch (error) {
            this.state.error = error;
            this.state.loading = false;
            this.handleError(error);
            throw error;
        }
    }
    
    /**
     * Validate component options
     */
    validateOptions() {
        if (!this.options.container) {
            throw new Error('Container element is required');
        }
        
        if (!(this.options.container instanceof HTMLElement)) {
            throw new Error('Container must be an HTMLElement');
        }
    }
    
    /**
     * Setup DOM structure
     */
    async setupDOM() {
        const container = this.options.container;
        container.innerHTML = '';
        container.className = 'component-container';
        
        // Create main elements
        this.elements = {
            wrapper: this.createElement('div', 'component-wrapper'),
            header: this.createElement('div', 'component-header'),
            content: this.createElement('div', 'component-content'),
            footer: this.createElement('div', 'component-footer')
        };
        
        // Assemble DOM
        this.elements.wrapper.appendChild(this.elements.header);
        this.elements.wrapper.appendChild(this.elements.content);
        this.elements.wrapper.appendChild(this.elements.footer);
        container.appendChild(this.elements.wrapper);
    }
    
    /**
     * Load required resources
     */
    async loadResources() {
        // Load data if provided
        if (this.options.data) {
            this.state.data = await this.processData(this.options.data);
        }
        
        // Load configuration
        if (this.options.config) {
            this.applyConfig(this.options.config);
        }
    }
    
    /**
     * Attach event listeners
     */
    attachEventListeners() {
        // Window resize
        const resizeHandler = this.debounce(() => this.handleResize(), 250);
        window.addEventListener('resize', resizeHandler);
        this.handlers.set('resize', resizeHandler);
        
        // Component-specific events
        this.setupComponentEvents();
    }
    
    /**
     * Setup component-specific events
     */
    setupComponentEvents() {
        // Override in subclasses
    }
    
    /**
     * Render the component
     */
    render() {
        if (!this.state.initialized) return;
        
        // Clear content
        this.elements.content.innerHTML = '';
        
        // Render based on state
        if (this.state.loading) {
            this.renderLoading();
        } else if (this.state.error) {
            this.renderError();
        } else if (this.state.data) {
            this.renderContent();
        } else {
            this.renderEmpty();
        }
    }
    
    /**
     * Render loading state
     */
    renderLoading() {
        this.elements.content.innerHTML = `
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Loading...</p>
            </div>
        `;
    }
    
    /**
     * Render error state
     */
    renderError() {
        this.elements.content.innerHTML = `
            <div class="error-state">
                <h3>Error</h3>
                <p>${this.state.error.message}</p>
                <button onclick="this.retry()">Retry</button>
            </div>
        `;
    }
    
    /**
     * Render empty state
     */
    renderEmpty() {
        this.elements.content.innerHTML = `
            <div class="empty-state">
                <p>No data available</p>
            </div>
        `;
    }
    
    /**
     * Render main content
     */
    renderContent() {
        // Override in subclasses
        this.elements.content.innerHTML = '<p>Content goes here</p>';
    }
    
    /**
     * Update component with new data
     */
    update(data) {
        this.state.data = data;
        this.render();
        this.emit('updated', data);
    }
    
    /**
     * Handle window resize
     */
    handleResize() {
        this.render();
        this.emit('resized');
    }
    
    /**
     * Handle errors
     */
    handleError(error) {
        console.error('Component error:', error);
        this.emit('error', error);
    }
    
    /**
     * Create DOM element
     */
    createElement(tag, className) {
        const element = document.createElement(tag);
        if (className) element.className = className;
        return element;
    }
    
    /**
     * Debounce function
     */
    debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }
    
    /**
     * Emit event
     */
    emit(event, data) {
        const handlers = this.subscriptions.filter(s => s.event === event);
        handlers.forEach(h => h.callback(data));
    }
    
    /**
     * Subscribe to events
     */
    on(event, callback) {
        this.subscriptions.push({ event, callback });
        return () => {
            const index = this.subscriptions.findIndex(
                s => s.event === event && s.callback === callback
            );
            if (index > -1) this.subscriptions.splice(index, 1);
        };
    }
    
    /**
     * Process data
     */
    async processData(data) {
        // Validate data
        if (!data) throw new Error('Invalid data');
        
        // Transform data
        return this.transformData(data);
    }
    
    /**
     * Transform data
     */
    transformData(data) {
        // Override in subclasses
        return data;
    }
    
    /**
     * Apply configuration
     */
    applyConfig(config) {
        this.options.config = { ...this.options.config, ...config };
        if (this.state.initialized) {
            this.render();
        }
    }
    
    /**
     * Destroy component
     */
    destroy() {
        // Remove event listeners
        this.handlers.forEach((handler, event) => {
            window.removeEventListener(event, handler);
        });
        this.handlers.clear();
        
        // Clear subscriptions
        this.subscriptions = [];
        
        // Clear cache
        this.cache.clear();
        
        // Clear DOM
        if (this.options.container) {
            this.options.container.innerHTML = '';
        }
        
        this.state.initialized = false;
        this.emit('destroyed');
    }
}
```

### Helper Functions

```javascript
/**
 * Utility functions for Frontend Project Setup
 */

/**
 * Format data for display
 */
export function formatData(data, options = {}) {
    if (!data) return null;
    
    const formatted = {
        ...data,
        timestamp: Date.now(),
        formatted: true
    };
    
    if (options.transform) {
        return options.transform(formatted);
    }
    
    return formatted;
}

/**
 * Validate input data
 */
export function validateInput(input, schema) {
    if (!input) {
        throw new Error('Input is required');
    }
    
    if (typeof input !== 'object') {
        throw new Error('Input must be an object');
    }
    
    if (schema) {
        // Validate against schema
        for (const [key, validator] of Object.entries(schema)) {
            if (!validator(input[key])) {
                throw new Error(`Invalid value for ${key}`);
            }
        }
    }
    
    return true;
}

/**
 * Deep clone object
 */
export function deepClone(obj) {
    if (obj === null || typeof obj !== 'object') return obj;
    if (obj instanceof Date) return new Date(obj.getTime());
    if (obj instanceof Array) return obj.map(item => deepClone(item));
    
    const cloned = {};
    for (const key in obj) {
        if (obj.hasOwnProperty(key)) {
            cloned[key] = deepClone(obj[key]);
        }
    }
    return cloned;
}

/**
 * Merge objects deeply
 */
export function deepMerge(target, ...sources) {
    if (!sources.length) return target;
    const source = sources.shift();
    
    if (isObject(target) && isObject(source)) {
        for (const key in source) {
            if (isObject(source[key])) {
                if (!target[key]) Object.assign(target, { [key]: {} });
                deepMerge(target[key], source[key]);
            } else {
                Object.assign(target, { [key]: source[key] });
            }
        }
    }
    
    return deepMerge(target, ...sources);
}

function isObject(item) {
    return item && typeof item === 'object' && !Array.isArray(item);
}

/**
 * Throttle function execution
 */
export function throttle(func, limit) {
    let inThrottle;
    return function(...args) {
        if (!inThrottle) {
            func.apply(this, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

/**
 * Create unique ID
 */
export function createId(prefix = 'id') {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}
```

## Advanced Features

### State Management

```javascript
/**
 * State manager for component state
 */
export class StateManager {
    constructor(initialState = {}) {
        this.state = initialState;
        this.listeners = [];
        this.history = [];
        this.maxHistory = 50;
    }
    
    getState() {
        return { ...this.state };
    }
    
    setState(updates, options = {}) {
        const prevState = this.getState();
        
        // Update state
        this.state = { ...this.state, ...updates };
        
        // Save to history
        if (!options.skipHistory) {
            this.history.push(prevState);
            if (this.history.length > this.maxHistory) {
                this.history.shift();
            }
        }
        
        // Notify listeners
        if (!options.silent) {
            this.notifyListeners(this.state, prevState);
        }
    }
    
    subscribe(listener) {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index > -1) this.listeners.splice(index, 1);
        };
    }
    
    notifyListeners(state, prevState) {
        this.listeners.forEach(listener => {
            try {
                listener(state, prevState);
            } catch (error) {
                console.error('Listener error:', error);
            }
        });
    }
    
    undo() {
        if (this.history.length > 0) {
            this.state = this.history.pop();
            this.notifyListeners(this.state, {});
        }
    }
    
    clearHistory() {
        this.history = [];
    }
}
```

### Event Bus

```javascript
/**
 * Event bus for component communication
 */
export class EventBus {
    constructor() {
        this.events = new Map();
    }
    
    on(event, callback) {
        if (!this.events.has(event)) {
            this.events.set(event, new Set());
        }
        this.events.get(event).add(callback);
        
        return () => this.off(event, callback);
    }
    
    off(event, callback) {
        if (this.events.has(event)) {
            this.events.get(event).delete(callback);
        }
    }
    
    emit(event, data) {
        if (this.events.has(event)) {
            this.events.get(event).forEach(callback => {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`Error in event handler for ${event}:`, error);
                }
            });
        }
    }
    
    clear() {
        this.events.clear();
    }
}
```

## Performance Optimization

### Memoization

```javascript
/**
 * Memoize expensive function calls
 */
export function memoize(fn, options = {}) {
    const cache = new Map();
    const maxSize = options.maxSize || 100;
    
    return function(...args) {
        const key = JSON.stringify(args);
        
        if (cache.has(key)) {
            return cache.get(key);
        }
        
        const result = fn.apply(this, args);
        
        cache.set(key, result);
        
        // Limit cache size
        if (cache.size > maxSize) {
            const firstKey = cache.keys().next().value;
            cache.delete(firstKey);
        }
        
        return result;
    };
}

/**
 * Example usage
 */
const expensiveOperation = memoize((data) => {
    // Expensive computation
    return data.map(item => item * 2).reduce((a, b) => a + b, 0);
});
```

### Virtual Scrolling

```javascript
/**
 * Virtual scrolling for large lists
 */
export class VirtualScroller {
    constructor(container, options) {
        this.container = container;
        this.options = {
            itemHeight: 50,
            buffer: 5,
            ...options
        };
        
        this.items = [];
        this.visibleRange = { start: 0, end: 0 };
        
        this.setupScrolling();
    }
    
    setItems(items) {
        this.items = items;
        this.updateVisibleRange();
        this.render();
    }
    
    setupScrolling() {
        this.container.addEventListener('scroll', () => {
            this.updateVisibleRange();
            this.render();
        });
    }
    
    updateVisibleRange() {
        const scrollTop = this.container.scrollTop;
        const containerHeight = this.container.clientHeight;
        
        const start = Math.max(0, 
            Math.floor(scrollTop / this.options.itemHeight) - this.options.buffer
        );
        const end = Math.min(this.items.length,
            Math.ceil((scrollTop + containerHeight) / this.options.itemHeight) + this.options.buffer
        );
        
        this.visibleRange = { start, end };
    }
    
    render() {
        const { start, end } = this.visibleRange;
        const visibleItems = this.items.slice(start, end);
        
        // Render only visible items
        this.container.innerHTML = visibleItems
            .map((item, index) => this.renderItem(item, start + index))
            .join('');
    }
    
    renderItem(item, index) {
        return `
            <div class="virtual-item" style="top: ${index * this.options.itemHeight}px">
                ${item}
            </div>
        `;
    }
}
```

## Testing

### Unit Tests

```javascript
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { MainComponent } from '../MainComponent.js';

describe('MainComponent', () => {
    let component;
    let container;

    beforeEach(() => {
        container = document.createElement('div');
        document.body.appendChild(container);
        
        component = new MainComponent({
            container,
            data: { test: 'data' }
        });
    });

    afterEach(() => {
        component.destroy();
        document.body.removeChild(container);
    });

    it('should initialize correctly', async () => {
        await component.initialize();
        expect(component.state.initialized).toBe(true);
    });

    it('should render content', async () => {
        await component.initialize();
        expect(container.innerHTML).not.toBe('');
    });

    it('should handle updates', async () => {
        await component.initialize();
        component.update({ new: 'data' });
        expect(component.state.data).toEqual({ new: 'data' });
    });

    it('should emit events', async () => {
        await component.initialize();
        
        let emitted = false;
        component.on('updated', () => {
            emitted = true;
        });
        
        component.update({ test: 'data' });
        expect(emitted).toBe(true);
    });

    it('should cleanup on destroy', () => {
        component.destroy();
        expect(component.state.initialized).toBe(false);
        expect(container.innerHTML).toBe('');
    });
});
```

### Integration Tests

```javascript
import { describe, it, expect } from 'vitest';
import { MainComponent } from '../MainComponent.js';
import { StateManager } from '../StateManager.js';
import { EventBus } from '../EventBus.js';

describe('Component Integration', () => {
    it('should integrate with state manager', async () => {
        const container = document.createElement('div');
        const state = new StateManager({ count: 0 });
        
        const component = new MainComponent({ container });
        await component.initialize();
        
        state.subscribe((newState) => {
            component.update(newState);
        });
        
        state.setState({ count: 1 });
        expect(component.state.data.count).toBe(1);
    });

    it('should integrate with event bus', async () => {
        const container = document.createElement('div');
        const eventBus = new EventBus();
        
        const component = new MainComponent({ container });
        await component.initialize();
        
        let received = false;
        eventBus.on('test-event', () => {
            received = true;
        });
        
        eventBus.emit('test-event');
        expect(received).toBe(true);
    });
});
```

## Styling

```css
/* Component styles */
.component-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
}

.component-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.component-header {
    padding: 1rem;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
}

.component-content {
    flex: 1;
    overflow: auto;
    padding: 1rem;
}

.component-footer {
    padding: 0.5rem 1rem;
    background: var(--color-bg-tertiary);
    border-top: 1px solid var(--color-border);
    font-size: 0.875rem;
}

/* Loading state */
.loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
}

.spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* Error state */
.error-state {
    text-align: center;
    padding: 2rem;
    color: var(--color-error);
}

.error-state button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
}

/* Empty state */
.empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-secondary);
}
```

## Best Practices

1. **Component Lifecycle**: Always initialize before use, cleanup on destroy
2. **Error Handling**: Catch and handle all errors gracefully
3. **Performance**: Use memoization, virtual scrolling for large datasets
4. **Testing**: Write unit and integration tests for all features
5. **Documentation**: Document public APIs and complex logic
6. **Accessibility**: Ensure keyboard navigation and screen reader support

## Common Pitfalls

1. **Memory Leaks**: Always remove event listeners and clear references
2. **State Mutations**: Never mutate state directly, always create new objects
3. **Async Errors**: Handle promise rejections properly
4. **Performance**: Avoid unnecessary re-renders and computations
5. **Testing**: Don't forget edge cases and error scenarios

## Debugging Tips

1. Use browser DevTools for debugging
2. Add logging at key points
3. Use breakpoints for complex logic
4. Test with different data sets
5. Monitor performance metrics
6. Check memory usage
7. Validate all inputs
8. Test error paths

## Key Takeaways

- Frontend Project Setup is essential for BorrowScope functionality
- Component-based architecture enables modularity
- State management ensures predictability
- Event-driven communication decouples components
- Performance optimization is critical for large datasets
- Comprehensive testing ensures reliability
- Proper cleanup prevents memory leaks

## Further Reading

- [Component Design Patterns](https://www.patterns.dev/)
- [JavaScript Best Practices](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)
- [Performance Optimization](https://web.dev/performance/)
- [Testing Strategies](https://martinfowler.com/testing/)
- [Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## Exercises

1. Implement additional features for the component
2. Add more comprehensive error handling
3. Optimize performance for large datasets
4. Write additional test cases
5. Extend the component with plugins
6. Add accessibility features
7. Implement internationalization
8. Create documentation

## Summary

This section covered frontend project setup with comprehensive examples, best practices, and testing strategies. The implementation provides a solid foundation for building production-ready features in BorrowScope. Key concepts include component architecture, state management, event handling, performance optimization, and testing.
