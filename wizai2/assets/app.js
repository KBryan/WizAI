class OpenSpecApp {
    constructor() {
        this.agentId = null;
        this.agents = [];
        this.skills = [];
        this.changes = [];
        this.specs = [];
        this.ws = null;
        this.retryCount = 0;
        this.maxRetries = 10;
        this.baseRetryDelay = 1000;
        this.retryTimer = null;
        this.isConnected = false;
        this.cliMode = false;
        this.commandPaletteOpen = false;
        this.slashPaletteOpen = false;
        this.slashCommands = [];
        this.filteredSlashCommands = [];
        this.selectedSlashIndex = 0;

        this.init();
    }

    async init() {
        await this.loadAgents();
        await this.loadSkills();
        this.buildSlashItems();
        this.loadChanges();
        this.loadSpecs();
        this.setupEventListeners();
        this.connectWebSocket();
        this.setupCommandPalette();
    }

    buildSlashItems() {
        console.log('[DEBUG] buildSlashItems called, skills:', this.skills.length);
        
        // Build skill items from loaded skills
        const skillItems = this.skills.map(skill => ({
            command: `/${skill.name.toLowerCase().replace(/\s+/g, '-')}`,
            description: skill.description || 'Available skill',
            category: 'Skills',
            icon: '🧠',
            example: `/${skill.name.toLowerCase().replace(/\s+/g, '-')}`
        }));

        // Built-in commands
        const builtInItems = [
            {
                command: '/ask',
                description: 'Ask the AI agent a question',
                category: 'General',
                icon: '💬',
                example: '/ask what is the market outlook?'
            },
            {
                command: '/code',
                description: 'Request code or software development help',
                category: 'Development',
                icon: '💻',
                example: '/code create a function'
            },
            {
                command: '/research',
                description: 'Research real estate market data',
                category: 'Research',
                icon: '🔍',
                example: '/research durham region'
            },
            {
                command: '/research:market',
                description: 'Conduct market research with ResearchLead',
                category: 'Research',
                icon: '📊',
                example: '/research:market analyze competitor landscape'
            },
            {
                command: '/research:technical',
                description: 'Technical research and feasibility study',
                category: 'Research',
                icon: '🔬',
                example: '/research:technical evaluate new technology'
            },
            {
                command: '/prd:create',
                description: 'Create Product Requirements Document',
                category: 'Research',
                icon: '📄',
                example: '/prd:create from research findings'
            },
            {
                command: '/prd:submit',
                description: 'Submit PRD for human approval',
                category: 'Research',
                icon: '✉️',
                example: '/prd:submit PRD-001'
            },
            {
                command: '/prd:handoff',
                description: 'Hand off approved PRD to development',
                category: 'Research',
                icon: '🤝',
                example: '/prd:handoff to software team'
            },
            {
                command: '/opsx:propose',
                description: 'Create a new change proposal',
                category: 'OpenSpec',
                icon: '📋',
                example: '/opsx:propose feature/user-authentication'
            },
            {
                command: '/opsx:apply',
                description: 'Apply an active change',
                category: 'OpenSpec',
                icon: '▶️',
                example: '/opsx:apply'
            },
            {
                command: '/opsx:archive',
                description: 'Archive a completed change',
                category: 'OpenSpec',
                icon: '✓',
                example: '/opsx:archive'
            }
        ];

        this.slashCommands = [...builtInItems, ...skillItems];
        this.slashPaletteOpen = false;
        this.selectedSlashIndex = 0;
        this.filteredSlashCommands = [];
        
        console.log('[DEBUG] Built', this.slashCommands.length, 'slash commands');
    }

    // -------------------------------------------------------------------------
    // Agent Management
    // -------------------------------------------------------------------------

    async loadAgents() {
        try {
            const response = await fetch('/api/agents');
            if (response.ok) {
                this.agents = await response.json();
                if (this.agents.length > 0) {
                    this.agentId = this.agents[0].id;
                } else {
                    // Create a default agent
                    await this.createDefaultAgent();
                }
            }
        } catch (e) {
            console.error('Failed to load agents:', e);
        }
    }

    async createDefaultAgent() {
        try {
            const response = await fetch('/api/agents', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: 'Default Agent',
                    role: 'openspec-executor'
                })
            });

            if (response.ok) {
                const agent = await response.json();
                this.agentId = agent.id;
                this.agents.push(agent);
            }
        } catch (e) {
            console.error('Failed to create default agent:', e);
        }
    }

    // -------------------------------------------------------------------------
    // Data Loading
    // -------------------------------------------------------------------------

    async loadSkills() {
        try {
            const response = await fetch('/api/skills');
            if (response.ok) {
                this.skills = await response.json();
            }
        } catch (e) {
            console.error('Failed to load skills:', e);
        }
    }

    loadChanges() {
        const changesList = document.getElementById('changes-list');
        if (!changesList) return;

        changesList.innerHTML = '<div class="loading-placeholder">Loading changes...</div>';

        setTimeout(() => {
            this.changes = [
                { id: '1', name: 'feature/user-auth', status: 'active', description: 'User authentication system' },
                { id: '2', name: 'fix/api-response', status: 'completed', description: 'API response formatting fix' }
            ];
            this.renderChanges();
        }, 500);
    }

    loadSpecs() {
        const specsList = document.getElementById('specs-list');
        if (!specsList) return;

        specsList.innerHTML = '<div class="loading-placeholder">Loading specs...</div>';

        setTimeout(() => {
            this.specs = [
                { id: '1', name: 'API Design', version: '1.0' },
                { id: '2', name: 'Database Schema', version: '2.1' }
            ];
            this.renderSpecs();
        }, 600);
    }

    async loadRealEstateData() {
        const container = document.getElementById('realestate-container');
        if (!container) return;

        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'stats' })
            });

            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data) {
                    container.innerHTML = this.renderRealEstateStats(result.data);
                } else {
                    container.innerHTML = '<p>No data available</p>';
                }
            } else {
                container.innerHTML = '<p>Failed to load data</p>';
            }
        } catch (e) {
            container.innerHTML = `<p>Error loading data: ${e.message}</p>`;
        }
    }

    async fetchRealEstateData() {
        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'stats' })
            });

            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data) {
                    return result.data;
                }
            }
            return null;
        } catch (e) {
            console.error('Failed to fetch real estate data:', e);
            return null;
        }
    }

    formatRealEstateResults(data) {
        const avgPrice = (data.avg_price || 0).toLocaleString();
        const medianPrice = (data.median_price || 0).toLocaleString();
        const pricePerSqft = data.price_per_sqft || 0;
        const activeListings = (data.active_listings || 0).toLocaleString();
        const daysOnMarket = data.avg_days_on_market || 0;
        const totalSales = (data.total_sales || 0).toLocaleString();

        const municipalities = this.formatMunicipalitiesHTML(data.municipality_breakdown);

        return `<article class="ai-response-card">
            <header class="ai-card-header">
                <div class="ai-avatar">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                        <line x1="9" y1="9" x2="15" y2="15"></line>
                        <line x1="15" y1="9" x2="9" y2="15"></line>
                    </svg>
                </div>
                <div class="ai-header-text">
                    <span class="ai-name">Research Assistant</span>
                    <time class="ai-timestamp">Just now</time>
                </div>
            </header>
            
            <div class="ai-card-body">
                <p class="ai-intro">Here's the latest market data for <strong>Durham Region</strong> based on current listings and sales data.</p>
                
                <div class="market-overview">
                    <h3>📊 Market Snapshot</h3>
                    <div class="metrics-row">
                        <div class="metric primary">
                            <span class="metric-value">$${avgPrice}</span>
                            <span class="metric-label">Average Price</span>
                        </div>
                        <div class="metric">
                            <span class="metric-value">$${medianPrice}</span>
                            <span class="metric-label">Median</span>
                        </div>
                        <div class="metric">
                            <span class="metric-value">${daysOnMarket}</span>
                            <span class="metric-label">Days on Market</span>
                        </div>
                    </div>
                    
                    <div class="metrics-grid">
                        <div class="metric-item">
                            <span class="item-value">$${pricePerSqft}</span>
                            <span class="item-label">Price per sq ft</span>
                        </div>
                        <div class="metric-item">
                            <span class="item-value">${activeListings}</span>
                            <span class="item-label">Active listings</span>
                        </div>
                        <div class="metric-item">
                            <span class="item-value">${totalSales}</span>
                            <span class="item-label">Monthly sales</span>
                        </div>
                    </div>
                </div>

                <div class="municipality-section">
                    <h3>🏘️ Municipalities by Average Price</h3>
                    <div class="muni-table">
                        ${municipalities}
                    </div>
                </div>

                <div class="market-insight">
                    <div class="insight-badge positive">
                        <span class="badge-icon">📈</span>
                        <div class="badge-content">
                            <strong>+5.2%</strong>
                            <span>Year-over-year growth</span>
                        </div>
                    </div>
                    <p class="insight-text">The Durham Region market shows steady growth with Whitby and Ajax leading in average prices. With only ${daysOnMarket} days on market, properties are moving quickly.</p>
                </div>
            </div>
            
            <footer class="ai-card-footer">
                <span class="data-source">Data: Durham Region Real Estate Board • March 2026</span>
            </footer>
        </article>`;
    }

    formatMunicipalities(breakdown) {
        if (!breakdown) return '';
        const sorted = Object.entries(breakdown)
            .sort((a, b) => (b[1].avg_price || 0) - (a[1].avg_price || 0));
        
        // Find the longest municipality name for alignment
        const maxLen = Math.max(...sorted.map(([name]) => name.length));
        
        return sorted.map(([name, data]) => {
            const price = (data.avg_price || 0).toLocaleString().padStart(9);
            const listings = (data.listings || 0).toLocaleString().padStart(3);
            const namePadded = name.padEnd(maxLen);
            return `  ${namePadded}  $${price}  (${listings} listings)`;
        }).join('\n');
    }

    formatMunicipalitiesHTML(breakdown) {
        if (!breakdown) return '';
        const sorted = Object.entries(breakdown)
            .sort((a, b) => (b[1].avg_price || 0) - (a[1].avg_price || 0));
        
        return sorted.map(([name, data], index) => {
            const price = (data.avg_price || 0).toLocaleString();
            const listings = data.listings || 0;
            const rank = index + 1;
            return `
                <div class="municipality-item">
                    <div class="muni-rank">#${rank}</div>
                    <div class="muni-name">${name}</div>
                    <div class="muni-price">$${price}</div>
                    <div class="muni-listings">${listings} listings</div>
                </div>
            `;
        }).join('');
    }

    // -------------------------------------------------------------------------
    // Event Listeners
    // -------------------------------------------------------------------------

    setupEventListeners() {
        // Send button and input
        const sendBtn = document.getElementById('send-btn');
        const messageInput = document.getElementById('message-input');

        if (sendBtn) {
            sendBtn.addEventListener('click', () => this.sendMessage());
        }

        if (messageInput) {
            messageInput.addEventListener('keydown', (e) => {
                if (this.slashPaletteOpen) {
                    this.handleSlashPaletteKeydown(e);
                    return;
                }

                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    this.sendMessage();
                }
            });
            messageInput.addEventListener('input', (e) => {
                this.autoResizeTextarea();
                this.handleSlashInput(e);
            });
            messageInput.addEventListener('keyup', (e) => {
                if (e.key === 'Escape') {
                    this.closeSlashPalette();
                }
            });
        }

        // Workflow buttons
        document.querySelectorAll('.workflow-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const command = btn.dataset.command;
                if (command) {
                    this.executeCommand(command);
                }
            });
        });

        // Quick actions
        document.querySelectorAll('.action-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const action = btn.dataset.action;
                this.handleQuickAction(action);
            });
        });

        // Real estate analyze button
        const realestateBtn = document.querySelector('[data-action="realestate-analyze"]');
        if (realestateBtn) {
            realestateBtn.addEventListener('click', () => {
                this.showModal('Real Estate Research',
                    '<div id="realestate-container"><p>Loading Durham Region market data...</p></div>');
                this.loadRealEstateData();
            });
        }

        // CLI toggle
        const cliBtn = document.getElementById('toggle-cli-btn');
        const closeCliBtn = document.getElementById('close-cli-btn');
        if (cliBtn) {
            cliBtn.addEventListener('click', () => this.toggleCli());
        }
        if (closeCliBtn) {
            closeCliBtn.addEventListener('click', () => this.toggleCli());
        }

        // CLI input
        const cliInput = document.getElementById('cli-input-field');
        if (cliInput) {
            cliInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    this.executeCliCommand(cliInput.value);
                    cliInput.value = '';
                }
            });
        }

        // Settings button
        const settingsBtn = document.getElementById('settings-btn');
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => {
                this.showModal('Settings', '<p>Settings panel coming soon...</p>');
            });
        }

        // Refresh changes
        const refreshBtn = document.getElementById('refresh-changes-btn');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.loadChanges());
        }

        // View specs
        const viewSpecsBtn = document.getElementById('view-specs-btn');
        if (viewSpecsBtn) {
            viewSpecsBtn.addEventListener('click', () => {
                this.showModal('Specifications', '<div id="specs-modal-content">Loading specs...</div>');
                this.renderSpecsModal();
            });
        }

        // Modal close
        const modalClose = document.getElementById('modal-close');
        const modalBackdrop = document.querySelector('.modal-backdrop');
        if (modalClose) {
            modalClose.addEventListener('click', () => this.closeModal());
        }
        if (modalBackdrop) {
            modalBackdrop.addEventListener('click', () => this.closeModal());
        }

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                e.preventDefault();
                this.openCommandPalette();
            }
            if (e.key === 'Escape') {
                this.closeCommandPalette();
                this.closeModal();
            }
        });
    }

    // -------------------------------------------------------------------------
    // WebSocket
    // -------------------------------------------------------------------------

    connectWebSocket() {
        if (this.retryCount >= this.maxRetries) {
            console.error('OpenSpecApp: maximum WebSocket reconnect attempts reached.');
            this._showError('Unable to connect to the server. Please refresh the page.');
            return;
        }

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;

        try {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                this.retryCount = 0;
                this.isConnected = true;
                this.updateConnectionStatus(true);
            };

            this.ws.onclose = () => {
                this.isConnected = false;
                this.updateConnectionStatus(false);
                this._scheduleReconnect();
            };

            this.ws.onerror = (err) => {
                console.error('WebSocket error:', err);
                this.isConnected = false;
                this.updateConnectionStatus(false);
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleWebSocketMessage(message);
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e);
                }
            };
        } catch (e) {
            console.error('WebSocket connection failed:', e);
            this._scheduleReconnect();
        }
    }

    _scheduleReconnect() {
        if (this.retryTimer) return;

        const delay = Math.min(
            this.baseRetryDelay * Math.pow(2, this.retryCount),
            30_000
        );
        this.retryCount++;

        console.info(`WebSocket reconnecting in ${delay}ms (attempt ${this.retryCount}/${this.maxRetries})`);

        this.retryTimer = setTimeout(() => {
            this.retryTimer = null;
            this.connectWebSocket();
        }, delay);
    }

    updateConnectionStatus(connected) {
        const statusEl = document.getElementById('connection-status');
        if (statusEl) {
            const dot = statusEl.querySelector('.status-dot');
            const text = statusEl.querySelector('.status-text');
            if (dot) dot.style.backgroundColor = connected ? '#10b981' : '#ef4444';
            if (text) text.textContent = connected ? 'Connected' : 'Disconnected';
        }
    }

    handleWebSocketMessage(message) {
        switch (message.type) {
            case 'task_update':
                this._onTaskUpdate(message);
                break;
            case 'task_result':
                this.addMessage(message.content, 'assistant');
                break;
            case 'error':
                this._showError(message.content ?? 'An unknown server error occurred.');
                break;
            default:
                console.warn('OpenSpecApp: unhandled WebSocket message type:', message.type, message);
        }
    }

    _onTaskUpdate(message) {
        console.info('Task update:', message);
    }

    // -------------------------------------------------------------------------
    // Messaging
    // -------------------------------------------------------------------------

    async sendMessage() {
        const input = document.getElementById('message-input');
        if (!input || !input.value.trim()) return;

        const message = input.value.trim();
        input.value = '';
        this.autoResizeTextarea();

        this.addMessage(message, 'user');

        // Handle slash commands locally
        if (message.startsWith('/')) {
            await this.handleCommand(message);
            return;
        }

        await this.sendToAgent(message);
    }

    async sendToAgent(message) {
        if (!this.agentId) {
            this._showError('No agent available. Please wait for initialization or refresh the page.');
            return;
        }

        this._setLoading(true);
        this.showLoadingIndicator();

        try {
            const response = await fetch('/api/tasks', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    agent_id: this.agentId,
                    task: message
                })
            });

            if (!response.ok) {
                const errorText = await response.text().catch(() => response.statusText);
                throw new Error(`Server error ${response.status}: ${errorText}`);
            }

            const result = await response.json();

            if (result.success) {
                this.addMessage(result.response, 'assistant');
            } else {
                this._showError(result.error ?? 'The request did not succeed.');
            }
        } catch (e) {
            console.error('Failed to send message:', e);
            this._showError('Failed to send your message. Please try again.');
        } finally {
            this._setLoading(false);
            this.hideLoadingIndicator();
        }
    }

    showLoadingIndicator() {
        const indicator = document.getElementById('loading-indicator');
        if (indicator) {
            indicator.style.display = 'flex';
            // Scroll to bottom to show the loading indicator
            const container = document.getElementById('chat-container');
            if (container) {
                container.scrollTop = container.scrollHeight;
            }
        }
    }

    hideLoadingIndicator() {
        const indicator = document.getElementById('loading-indicator');
        if (indicator) {
            indicator.style.display = 'none';
        }
    }

    _setLoading(isLoading) {
        const sendBtn = document.getElementById('send-btn');
        const input = document.getElementById('message-input');

        if (sendBtn) {
            sendBtn.disabled = isLoading;
            sendBtn.innerHTML = isLoading ? 'Sending...' : `Send <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M2 7h10M8 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
        }
        if (input) {
            input.disabled = isLoading;
        }
    }

    addMessage(content, role) {
        const container = document.getElementById('chat-container');
        if (!container) return;

        const welcome = container.querySelector('.welcome-message');
        if (welcome) welcome.remove();

        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${role}`;

        if (role === 'assistant') {
            // Check if content contains HTML (structured card)
            const hasHTML = content.includes('<') && content.includes('>');
            
            if (hasHTML) {
                // Structured card output (keeps existing ai-response-card)
                messageDiv.innerHTML = `
                    <div class="message-avatar">AI</div>
                    <div class="message-content">${content}</div>
                `;
            } else {
                // ChatGPT-style conversational layout with markdown
                const contentId = 'msg-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
                const formattedContent = this.formatMarkdown(content);
                messageDiv.innerHTML = `
                    <div class="message-avatar">AI</div>
                    <div class="message-content">
                        <div class="assistant-meta">
                            <span class="assistant-name">WizAI</span>
                        </div>
                        <div class="assistant-body" id="${contentId}">${formattedContent}</div>
                        <div class="assistant-actions">
                            <button onclick="navigator.clipboard.writeText(document.getElementById('${contentId}').innerText)">Copy</button>
                            <button onclick="app.regenerateResponse()">Regenerate</button>
                        </div>
                    </div>
                `;
            }
        } else {
            // User messages - simple bubble
            messageDiv.innerHTML = `<div class="message-content">${this.escapeHtml(content)}</div>`;
        }

        container.appendChild(messageDiv);
        container.scrollTop = container.scrollHeight;
    }

    formatMarkdown(text) {
        if (!text) return '';
        
        // Process line by line for better control
        const lines = text.split('\n');
        let html = [];
        let inList = false;
        let listType = '';
        
        for (let i = 0; i < lines.length; i++) {
            let line = lines[i];
            
            // Skip empty lines but preserve spacing
            if (line.trim() === '') {
                if (inList) {
                    html.push(listType === 'ul' ? '</ul>' : '</ol>');
                    inList = false;
                }
                html.push('<br>');
                continue;
            }
            
            // Check for list items
            const unorderedMatch = line.match(/^\- (.*)$/);
            const orderedMatch = line.match(/^\d+\. (.*)$/);
            
            if (unorderedMatch) {
                if (!inList || listType !== 'ul') {
                    if (inList) html.push('</ol>');
                    html.push('<ul>');
                    inList = true;
                    listType = 'ul';
                }
                html.push(`<li>${this.processInlineMarkdown(unorderedMatch[1])}</li>`);
                continue;
            }
            
            if (orderedMatch) {
                if (!inList || listType !== 'ol') {
                    if (inList) html.push('</ul>');
                    html.push('<ol>');
                    inList = true;
                    listType = 'ol';
                }
                html.push(`<li>${this.processInlineMarkdown(orderedMatch[1])}</li>`);
                continue;
            }
            
            // Close list if we're not in a list item
            if (inList) {
                html.push(listType === 'ul' ? '</ul>' : '</ol>');
                inList = false;
            }
            
            // Headers
            if (line.match(/^### (.*$)/)) {
                html.push(line.replace(/^### (.*$)/, '<h3>$1</h3>'));
            } else if (line.match(/^## (.*$)/)) {
                html.push(line.replace(/^## (.*$)/, '<h2>$1</h2>'));
            } else if (line.match(/^# (.*$)/)) {
                html.push(line.replace(/^# (.*$)/, '<h1>$1</h1>'));
            }
            // Blockquotes
            else if (line.match(/^> (.*$)/)) {
                html.push(line.replace(/^> (.*$)/, '<blockquote>$1</blockquote>'));
            }
            // Regular paragraph with inline formatting
            else {
                html.push(`<p>${this.processInlineMarkdown(line)}</p>`);
            }
        }
        
        // Close any open list
        if (inList) {
            html.push(listType === 'ul' ? '</ul>' : '</ol>');
        }
        
        return html.join('\n');
    }
    
    processInlineMarkdown(text) {
        // First escape HTML to prevent XSS, but use placeholders for markdown
        let processed = text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
        
        // Now convert markdown to HTML (safe because we escaped first)
        return processed
            // Bold
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            // Italic  
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            // Inline code
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            // Links
            .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');
    }

    regenerateResponse() {
        // Placeholder for regenerate functionality
        this.addMessage('Regenerating response... (feature coming soon)', 'assistant');
    }

    _showError(message) {
        const container = document.getElementById('chat-container');
        if (!container) {
            console.error('UI error (no container):', message);
            return;
        }

        const errorDiv = document.createElement('div');
        errorDiv.className = 'message error';

        const contentDiv = document.createElement('div');
        contentDiv.className = 'message-content';
        contentDiv.textContent = message;

        errorDiv.appendChild(contentDiv);
        container.appendChild(errorDiv);
        container.scrollTop = container.scrollHeight;
    }

    // -------------------------------------------------------------------------
    // Commands & Actions
    // -------------------------------------------------------------------------

    executeCommand(command) {
        const input = document.getElementById('message-input');
        if (input) {
            input.value = command + ' ';
            input.focus();
            this.autoResizeTextarea();
        }
    }

    async handleCommand(command) {
        const parts = command.split(' ');
        const cmd = parts[0];
        const args = parts.slice(1).join(' ');

        // Check if command matches a loaded skill
        const matchedSkill = this.skills.find(skill =>
            `/${skill.name.toLowerCase().replace(/\s+/g, '-')}` === cmd
        );

        if (matchedSkill) {
            await this.sendToAgent(`Use skill "${matchedSkill.name}" ${args}`.trim());
            return;
        }

        switch (cmd) {
            case '/opsx:propose':
                this.addMessage(`Creating proposal: ${args || 'new change'}...`, 'assistant');
                break;
            case '/opsx:apply':
                this.addMessage('Applying active change...', 'assistant');
                break;
            case '/opsx:archive':
                this.addMessage('Archiving completed change...', 'assistant');
                break;
            case '/research':
                console.log(`[DEBUG] /research command received with args: "${args}"`);
                this.addMessage(`Researching: ${args || 'real estate market'}...`, 'assistant');
                
                const location = args.toLowerCase();
                console.log(`[DEBUG] Processing location: "${location}"`);
                
                if (location.includes('durham')) {
                    console.log('[DEBUG] Matched Durham region, fetching data...');
                    this.showLoadingIndicator();
                    try {
                        const data = await this.fetchRealEstateData();
                        console.log('[DEBUG] Data received:', data ? 'YES' : 'NO');
                        if (data) {
                            this.addMessage(this.formatRealEstateResults(data), 'assistant');
                        } else {
                            this.addMessage('⚠️ No data available for Durham Region', 'assistant');
                        }
                    } finally {
                        this.hideLoadingIndicator();
                    }
                } else if (location.includes('toronto')) {
                    console.log('[DEBUG] Matched Toronto - data not yet available');
                    this.addMessage('🏙️ Toronto market data coming soon!\n\nCurrently available:\n• Durham Region (Ontario)\n\nTry: /research durham region', 'assistant');
                } else {
                    console.log(`[DEBUG] No handler for location: "${location}"`);
                    this.addMessage(`📍 Location "${args}" not yet supported.\n\nCurrently available:\n• Durham Region (Ontario)\n\nTry: /research durham region`, 'assistant');
                }
                break;
            case '/research:market':
                this.addMessage(`🔬 Initiating market research: ${args || 'general market analysis'}`, 'assistant');
                await this.sendToAgent(`As ResearchLead, conduct market research on: ${args}`);
                break;
            case '/research:technical':
                this.addMessage(`🔬 Initiating technical research: ${args || 'feasibility study'}`, 'assistant');
                await this.sendToAgent(`As ResearchLead, conduct technical research on: ${args}`);
                break;
            case '/prd:create':
                this.addMessage(`📄 Creating PRD: ${args || 'from research findings'}`, 'assistant');
                await this.sendToAgent(`As ResearchLead, create a PRD for: ${args}`);
                break;
            case '/prd:submit':
                this.addMessage(`📤 Submitting PRD for human approval: ${args || 'current PRD'}`, 'assistant');
                await this.sendToAgent(`As ResearchLead, submit PRD ${args} for approval`);
                break;
            case '/prd:handoff':
                this.addMessage(`🤝 Handing off to development: ${args || 'approved PRD'}`, 'assistant');
                await this.sendToAgent(`As ResearchLead, hand off PRD to development: ${args}`);
                break;
            case '/code':
                if (!args) {
                    this.addMessage('Please provide a coding task after /code', 'assistant');
                    break;
                }
                await this.sendToAgent(`Code: ${args}`);
                break;
            case '/ask':
                if (!args) {
                    this.addMessage('Please provide a question after /ask', 'assistant');
                    break;
                }
                // Send the question to the agent
                await this.sendToAgent(args);
                break;
            default:
                this.addMessage(`Unknown command: ${cmd}`, 'assistant');
        }
    }

    handleQuickAction(action) {
        switch (action) {
            case 'agents':
                this.showAgentsModal();
                break;
            case 'skills':
                this.showSkillsModal();
                break;
            case 'agents-md':
                this.showModal('AGENTS.md', '<p>AGENTS.md documentation will be displayed here.</p>');
                break;
        }
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    renderChanges() {
        const changesList = document.getElementById('changes-list');
        if (!changesList) return;

        if (this.changes.length === 0) {
            changesList.innerHTML = '<div class="empty-state">No active changes</div>';
            return;
        }

        changesList.innerHTML = this.changes.map(change => `
            <div class="change-item ${change.status}">
                <div class="change-name">${change.name}</div>
                <div class="change-desc">${change.description}</div>
                <div class="change-status">${change.status}</div>
            </div>
        `).join('');
    }

    renderSpecs() {
        const specsList = document.getElementById('specs-list');
        if (!specsList) return;

        if (this.specs.length === 0) {
            specsList.innerHTML = '<div class="empty-state">No specs available</div>';
            return;
        }

        specsList.innerHTML = this.specs.map(spec => `
            <div class="spec-item">
                <div class="spec-name">${spec.name}</div>
                <div class="spec-version">v${spec.version}</div>
            </div>
        `).join('');
    }

    renderSpecsModal() {
        const container = document.getElementById('specs-modal-content');
        if (!container) return;

        if (this.specs.length === 0) {
            container.innerHTML = '<p>No specifications available</p>';
            return;
        }

        container.innerHTML = `
            <div class="specs-grid">
                ${this.specs.map(spec => `
                    <div class="spec-card">
                        <h4>${spec.name}</h4>
                        <span class="version">v${spec.version}</span>
                    </div>
                `).join('')}
            </div>
        `;
    }

    renderRealEstateStats(data) {
        return `
            <div class="realestate-stats">
                <h3>Durham Region Real Estate Statistics</h3>
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-value">$${(data.avg_price || 0).toLocaleString()}</div>
                        <div class="stat-label">Average Price</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">$${(data.median_price || 0).toLocaleString()}</div>
                        <div class="stat-label">Median Price</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${data.active_listings || 0}</div>
                        <div class="stat-label">Active Listings</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${data.avg_days_on_market || 0}</div>
                        <div class="stat-label">Days on Market</div>
                    </div>
                </div>
            </div>
        `;
    }

    // -------------------------------------------------------------------------
    // Modals
    // -------------------------------------------------------------------------

    showModal(title, content) {
        const modal = document.getElementById('modal');
        const modalTitle = document.getElementById('modal-title');
        const modalBody = document.getElementById('modal-body');

        if (modal && modalTitle && modalBody) {
            modalTitle.textContent = title;
            modalBody.innerHTML = content;
            modal.classList.add('active');
        }
    }

    closeModal() {
        const modal = document.getElementById('modal');
        if (modal) {
            modal.classList.remove('active');
        }
    }

    showAgentsModal() {
        const content = this.agents.length > 0
            ? `<div class="agents-list">${this.agents.map(a => `
                <div class="agent-card">
                    <h4>${a.name}</h4>
                    <span class="role">${a.role}</span>
                    <span class="status ${a.status.toLowerCase()}">${a.status}</span>
                </div>
            `).join('')}</div>`
            : '<p>No agents available</p>';

        this.showModal('Agents', content);
    }

    showSkillsModal() {
        const content = this.skills.length > 0
            ? `<div class="skills-list">${this.skills.map(s => `
                <div class="skill-card">
                    <h4>${s.name}</h4>
                    <p>${s.description}</p>
                </div>
            `).join('')}</div>`
            : '<p>No skills available</p>';

        this.showModal('Skills', content);
    }

    // -------------------------------------------------------------------------
    // CLI Panel
    // -------------------------------------------------------------------------

    toggleCli() {
        this.cliMode = !this.cliMode;
        const cliPanel = document.getElementById('cli-panel');
        if (cliPanel) {
            cliPanel.classList.toggle('active', this.cliMode);
        }
    }

    executeCliCommand(command) {
        if (!command.trim()) return;

        const output = document.getElementById('cli-output');
        if (output) {
            const line = document.createElement('div');
            line.className = 'cli-line';
            line.innerHTML = `<span class="cli-prompt">$</span> <span class="cli-command">${this.escapeHtml(command)}</span>`;
            output.appendChild(line);

            const response = document.createElement('div');
            response.className = 'cli-response';
            response.textContent = `Executed: ${command}`;
            output.appendChild(response);

            output.scrollTop = output.scrollHeight;
        }
    }

    // -------------------------------------------------------------------------
    // Command Palette
    // -------------------------------------------------------------------------

    setupCommandPalette() {
        const palette = document.getElementById('command-palette');
        const input = document.getElementById('palette-input');
        const results = document.getElementById('palette-results');

        if (!palette || !input || !results) return;

        const commands = [
            { name: 'Propose Change', shortcut: 'Ctrl+P', action: () => this.executeCommand('/opsx:propose') },
            { name: 'Apply Change', shortcut: 'Ctrl+A', action: () => this.executeCommand('/opsx:apply') },
            { name: 'Archive Change', shortcut: 'Ctrl+R', action: () => this.executeCommand('/opsx:archive') },
            { name: 'View Agents', shortcut: '', action: () => this.handleQuickAction('agents') },
            { name: 'Load Skills', shortcut: '', action: () => this.handleQuickAction('skills') },
            { name: 'Toggle CLI', shortcut: '', action: () => this.toggleCli() },
        ];

        input.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();
            const filtered = commands.filter(c => c.name.toLowerCase().includes(query));
            results.innerHTML = filtered.map((c, i) => `
                <div class="palette-item ${i === 0 ? 'selected' : ''}" data-index="${i}">
                    <span class="name">${c.name}</span>
                    <span class="shortcut">${c.shortcut}</span>
                </div>
            `).join('');

            results.querySelectorAll('.palette-item').forEach((item, idx) => {
                item.addEventListener('click', () => {
                    filtered[idx].action();
                    this.closeCommandPalette();
                });
            });
        });

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                const selected = results.querySelector('.palette-item.selected');
                if (selected) {
                    const index = parseInt(selected.dataset.index);
                    const query = input.value.toLowerCase();
                    const filtered = commands.filter(c => c.name.toLowerCase().includes(query));
                    if (filtered[index]) {
                        filtered[index].action();
                        this.closeCommandPalette();
                    }
                }
            }
        });

        const backdrop = palette.querySelector('.palette-backdrop');
        if (backdrop) {
            backdrop.addEventListener('click', () => this.closeCommandPalette());
        }
    }

    openCommandPalette() {
        const palette = document.getElementById('command-palette');
        const input = document.getElementById('palette-input');
        if (palette) {
            palette.classList.add('active');
            this.commandPaletteOpen = true;
            if (input) {
                input.value = '';
                input.focus();
                input.dispatchEvent(new Event('input'));
            }
        }
    }

    closeCommandPalette() {
        const palette = document.getElementById('command-palette');
        if (palette) {
            palette.classList.remove('active');
            this.commandPaletteOpen = false;
        }
    }

    // -------------------------------------------------------------------------
    // Slash Command Palette
    // -------------------------------------------------------------------------
    // Slash Command Palette
    // -------------------------------------------------------------------------

    handleSlashInput(e) {
        const input = e.target;
        const value = input.value;
        
        console.log('[DEBUG] Input value:', value, 'slashPaletteOpen:', this.slashPaletteOpen);
        
        // Check if user just typed "/"
        if (value === '/') {
            console.log('[DEBUG] Opening slash palette');
            this.openSlashPalette();
        } else if (this.slashPaletteOpen && value.startsWith('/')) {
            // Filter commands based on what's typed after "/"
            const query = value.slice(1).toLowerCase();
            console.log('[DEBUG] Filtering with query:', query);
            this.filteredSlashCommands = this.slashCommands.filter(cmd => 
                cmd.command.toLowerCase().includes(query) ||
                cmd.description.toLowerCase().includes(query) ||
                cmd.category.toLowerCase().includes(query)
            );
            this.selectedSlashIndex = 0;
            this.renderSlashPalette();
        } else if (this.slashPaletteOpen && !value.startsWith('/')) {
            // User deleted the "/", close the palette
            console.log('[DEBUG] Closing slash palette');
            this.closeSlashPalette();
        }
    }

    handleSlashPaletteKeydown(e) {
        if (!this.slashPaletteOpen) return;
        
        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                this.selectedSlashIndex = (this.selectedSlashIndex + 1) % this.filteredSlashCommands.length;
                this.renderSlashPalette();
                break;
            case 'ArrowUp':
                e.preventDefault();
                this.selectedSlashIndex = (this.selectedSlashIndex - 1 + this.filteredSlashCommands.length) % this.filteredSlashCommands.length;
                this.renderSlashPalette();
                break;
            case 'Enter':
                e.preventDefault();
                if (this.filteredSlashCommands[this.selectedSlashIndex]) {
                    this.selectSlashCommand(this.filteredSlashCommands[this.selectedSlashIndex]);
                }
                break;
            case 'Escape':
                e.preventDefault();
                this.closeSlashPalette();
                break;
            case 'Tab':
                e.preventDefault();
                if (this.filteredSlashCommands[this.selectedSlashIndex]) {
                    this.selectSlashCommand(this.filteredSlashCommands[this.selectedSlashIndex]);
                }
                break;
        }
    }

    openSlashPalette() {
        console.log('[DEBUG] openSlashPalette called');
        console.log('[DEBUG] slashCommands length:', this.slashCommands.length);
        this.slashPaletteOpen = true;
        this.filteredSlashCommands = [...this.slashCommands];
        this.selectedSlashIndex = 0;
        this.renderSlashPalette();
        
        const palette = document.getElementById('slash-palette');
        console.log('[DEBUG] slash-palette element:', palette);
        if (palette) {
            palette.style.display = 'block';
            console.log('[DEBUG] Slash palette opened');
        } else {
            console.error('[DEBUG] slash-palette element not found!');
        }
    }

    closeSlashPalette() {
        this.slashPaletteOpen = false;
        const palette = document.getElementById('slash-palette');
        if (palette) {
            palette.style.display = 'none';
        }
    }

    renderSlashPalette() {
        console.log('[DEBUG] renderSlashPalette called');
        const container = document.getElementById('slash-palette-items');
        console.log('[DEBUG] slash-palette-items element:', container);
        if (!container) {
            console.error('[DEBUG] slash-palette-items element not found!');
            return;
        }
        
        if (this.filteredSlashCommands.length === 0) {
            container.innerHTML = '<div class="slash-no-results">No commands found</div>';
            return;
        }
        
        // Group by category
        const grouped = this.filteredSlashCommands.reduce((acc, cmd) => {
            if (!acc[cmd.category]) acc[cmd.category] = [];
            acc[cmd.category].push(cmd);
            return acc;
        }, {});
        
        let html = '';
        let itemIndex = 0;
        
        Object.entries(grouped).forEach(([category, commands]) => {
            html += `<div class="slash-category">${category}</div>`;
            commands.forEach(cmd => {
                const isSelected = itemIndex === this.selectedSlashIndex;
                html += `
                    <div class="slash-item ${isSelected ? 'selected' : ''}" data-index="${itemIndex}" data-command="${cmd.command}">
                        <div class="slash-item-icon">${cmd.icon}</div>
                        <div class="slash-item-content">
                            <div class="slash-item-command">${cmd.command}</div>
                            <div class="slash-item-desc">${cmd.description}</div>
                        </div>
                        <div class="slash-item-category">${category}</div>
                    </div>
                `;
                itemIndex++;
            });
        });
        
        console.log('[DEBUG] Generated HTML length:', html.length);
        console.log('[DEBUG] Generated HTML preview:', html.substring(0, 200));
        container.innerHTML = html;
        console.log('[DEBUG] HTML inserted, container now has', container.children.length, 'children');
        console.log('[DEBUG] Container innerHTML:', container.innerHTML.substring(0, 300));
        
        // Add click handlers
        container.querySelectorAll('.slash-item').forEach(item => {
            item.addEventListener('click', () => {
                const index = parseInt(item.dataset.index);
                this.selectSlashCommand(this.filteredSlashCommands[index]);
            });
        });
    }

    selectSlashCommand(cmd) {
        const input = document.getElementById('message-input');
        if (input) {
            input.value = cmd.example || cmd.command + ' ';
            input.focus();
        }
        this.closeSlashPalette();
    }

    // -------------------------------------------------------------------------
    // Utilities
    // -------------------------------------------------------------------------

    autoResizeTextarea() {
        const textarea = document.getElementById('message-input');
        if (textarea) {
            textarea.style.height = 'auto';
            textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
        }
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.app = new OpenSpecApp();
});
