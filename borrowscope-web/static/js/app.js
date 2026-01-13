// BorrowScope Web - Client Application
(function() {
    'use strict';

    const API_BASE = window.location.origin;
    
    // Fetch and display server status
    async function checkHealth() {
        try {
            const response = await fetch(`${API_BASE}/health`);
            const data = await response.json();
            
            const statusEl = document.getElementById('server-status');
            if (response.ok && data.status === 'healthy') {
                statusEl.textContent = '✓ Online';
                statusEl.classList.remove('loading', 'error');
                statusEl.classList.add('success');
            } else {
                throw new Error('Server unhealthy');
            }
        } catch (error) {
            const statusEl = document.getElementById('server-status');
            statusEl.textContent = '✗ Offline';
            statusEl.classList.remove('loading', 'success');
            statusEl.classList.add('error');
            console.error('Health check failed:', error);
        }
    }

    // Fetch and display app info
    async function loadAppInfo() {
        try {
            const response = await fetch(`${API_BASE}/api/info`);
            const data = await response.json();
            
            document.getElementById('app-version').textContent = data.version;
            document.getElementById('app-version').classList.remove('loading');
            
            document.getElementById('app-env').textContent = data.environment;
            document.getElementById('app-env').classList.remove('loading');
        } catch (error) {
            document.getElementById('app-version').textContent = 'Error';
            document.getElementById('app-version').classList.add('error');
            document.getElementById('app-env').textContent = 'Error';
            document.getElementById('app-env').classList.add('error');
            console.error('Failed to load app info:', error);
        }
    }

    // Initialize on page load
    document.addEventListener('DOMContentLoaded', function() {
        console.log('BorrowScope Web initialized');
        
        // Load initial data
        checkHealth();
        loadAppInfo();
        
        // Refresh health status every 30 seconds
        setInterval(checkHealth, 30000);
    });

    // Handle API link clicks with fetch preview
    document.querySelectorAll('.api-link').forEach(link => {
        link.addEventListener('click', async function(e) {
            if (e.ctrlKey || e.metaKey) return; // Allow normal navigation with Ctrl/Cmd
            
            e.preventDefault();
            const url = this.getAttribute('href');
            
            try {
                const response = await fetch(`${API_BASE}${url}`);
                const contentType = response.headers.get('content-type');
                
                if (contentType && contentType.includes('application/json')) {
                    const data = await response.json();
                    console.log(`Response from ${url}:`, data);
                    alert(`Response from ${url}:\n\n${JSON.stringify(data, null, 2)}`);
                } else {
                    window.location.href = url;
                }
            } catch (error) {
                console.error(`Failed to fetch ${url}:`, error);
                alert(`Error fetching ${url}: ${error.message}`);
            }
        });
    });

    // Expose API for console debugging
    window.BorrowScope = {
        checkHealth,
        loadAppInfo,
        version: '0.1.0'
    };
})();
