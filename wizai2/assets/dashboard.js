// Market Analysis Dashboard
// Professional dashboard for real estate and other market analysis

class MarketDashboard {
    constructor() {
        this.charts = {};
        this.currentRegion = 'durham';
        this.data = {
            stats: {},
            trends: [],
            municipalities: [],
            recentSales: []
        };
    }

    init() {
        console.log('MarketDashboard initializing...');
        this.setupEventListeners();
        
        // Only initialize charts if Chart.js is available
        if (typeof Chart !== 'undefined') {
            this.initializeCharts();
        } else {
            console.warn('Chart.js not loaded, skipping chart initialization');
        }
        
        this.loadDashboardData();
    }

    setupEventListeners() {
        console.log('Setting up event listeners...');

        // Fetch data button
        const fetchBtn = document.getElementById('fetch-data-btn');
        if (fetchBtn) {
            console.log('fetch-data-btn found, attaching listener');
            fetchBtn.onclick = (e) => {
                console.log('Fetch button clicked!');
                e.preventDefault();
                this.fetchData();
            };
        } else {
            console.warn('fetch-data-btn not found');
        }

        // Export button
        const exportBtn = document.getElementById('export-btn');
        if (exportBtn) {
            exportBtn.onclick = () => this.exportReport();
        }

        // Region selector
        const regionSelector = document.getElementById('region-selector');
        if (regionSelector) {
            regionSelector.onchange = (e) => {
                this.currentRegion = e.target.value;
                this.loadDashboardData();
            };
        }

        // Chart period buttons
        document.querySelectorAll('.chart-btn').forEach(btn => {
            btn.onclick = (e) => {
                document.querySelectorAll('.chart-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                this.updatePriceChart(e.target.dataset.period);
            };
        });

        // Table search
        const tableSearch = document.getElementById('table-search');
        if (tableSearch) {
            tableSearch.oninput = (e) => this.filterTable(e.target.value);
        }

        // Property filter
        const propertyFilter = document.getElementById('property-filter');
        if (propertyFilter) {
            propertyFilter.onchange = (e) => this.filterByPropertyType(e.target.value);
        }

        console.log('Event listeners setup complete');
    }

    initializeCharts() {
        console.log('Initializing charts...');

        try {
            // Price Trends Chart
            const priceCanvas = document.getElementById('price-trends-chart');
            if (priceCanvas) {
                this.charts.priceTrends = new Chart(priceCanvas.getContext('2d'), {
                    type: 'line',
                    data: {
                        labels: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'],
                        datasets: [{
                            label: 'Average Price',
                            data: [820000, 835000, 840000, 845000, 850000, 850000],
                            borderColor: '#6366f1',
                            backgroundColor: 'rgba(99, 102, 241, 0.1)',
                            borderWidth: 3,
                            fill: true,
                            tension: 0.4
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: { legend: { display: false } },
                        scales: {
                            y: {
                                beginAtZero: false,
                                ticks: { callback: (v) => '$' + (v / 1000) + 'k', color: '#6b7280' },
                                grid: { color: '#2a2a35' }
                            },
                            x: { ticks: { color: '#6b7280' }, grid: { display: false } }
                        }
                    }
                });
            }

            // Municipality Chart
            const munCanvas = document.getElementById('municipality-chart');
            if (munCanvas) {
                this.charts.municipality = new Chart(munCanvas.getContext('2d'), {
                    type: 'bar',
                    data: {
                        labels: ['Ajax', 'Pickering', 'Oshawa', 'Whitby'],
                        datasets: [{
                            label: 'Avg Price',
                            data: [925000, 875000, 725000, 950000],
                            backgroundColor: ['#6366f1', '#8b5cf6', '#ec4899', '#14b8a6'],
                            borderRadius: 6
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: { legend: { display: false } },
                        scales: {
                            y: { ticks: { callback: (v) => '$' + (v / 1000) + 'k', color: '#6b7280' }, grid: { color: '#2a2a35' } },
                            x: { ticks: { color: '#6b7280' }, grid: { display: false } }
                        }
                    }
                });
            }

            // Property Type Chart
            const propCanvas = document.getElementById('property-type-chart');
            if (propCanvas) {
                this.charts.propertyType = new Chart(propCanvas.getContext('2d'), {
                    type: 'doughnut',
                    data: {
                        labels: ['Detached', 'Semi', 'Townhouse', 'Condo'],
                        datasets: [{ data: [45, 15, 25, 15], backgroundColor: ['#6366f1', '#8b5cf6', '#ec4899', '#14b8a6'], borderWidth: 0 }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: { legend: { position: 'bottom', labels: { color: '#a0a0b0', padding: 20 } } }
                    }
                });
            }

            // Inventory Chart
            const invCanvas = document.getElementById('inventory-chart');
            if (invCanvas) {
                this.charts.inventory = new Chart(invCanvas.getContext('2d'), {
                    type: 'line',
                    data: {
                        labels: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'],
                        datasets: [{
                            label: 'Active Listings',
                            data: [1400, 1350, 1300, 1280, 1250, 1247],
                            borderColor: '#14b8a6',
                            backgroundColor: 'rgba(20, 184, 166, 0.1)',
                            borderWidth: 3,
                            fill: true,
                            tension: 0.4
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: { legend: { display: false } },
                        scales: {
                            y: { ticks: { color: '#6b7280' }, grid: { color: '#2a2a35' } },
                            x: { ticks: { color: '#6b7280' }, grid: { display: false } }
                        }
                    }
                });
            }

            console.log('Charts initialized');
        } catch (error) {
            console.error('Chart initialization error:', error);
        }
    }

    async loadDashboardData() {
        console.log('Loading dashboard data...');
        this.showLoading();
        
        try {
            const response = await fetch('/api/realestate/status');
            const result = await response.json();
            console.log('Status:', result);
            
            if (result.success) {
                await this.loadStats();
                await this.loadTrends();
                await this.loadRecentSales();
                this.showToast('Dashboard updated', 'success');
            } else {
                this.loadDemoData();
                this.showToast('Using demo data', 'warning');
            }
        } catch (error) {
            console.error('Load error:', error);
            this.loadDemoData();
            this.showToast('Using demo data', 'error');
        } finally {
            this.hideLoading();
        }
    }

    async loadStats() {
        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'stats', parameters: { days: 30 } })
            });
            const result = await response.json();
            if (result.success && result.data) {
                this.updateStats(result.data);
            }
        } catch (error) {
            console.error('Stats error:', error);
        }
    }

    async loadTrends() {
        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'trends', parameters: { months: 6 } })
            });
            const result = await response.json();
            if (result.success && result.data) {
                this.updateTrendsChart(result.data);
            }
        } catch (error) {
            console.error('Trends error:', error);
        }
    }

    async loadRecentSales() {
        const demoSales = [
            { address: '123 Sample St', municipality: 'Oshawa', type: 'Detached', price: 725000, date: '2024-03-15', days: 8 },
            { address: '456 Example Ave', municipality: 'Ajax', type: 'Semi', price: 825000, date: '2024-03-14', days: 12 },
            { address: '789 Demo Rd', municipality: 'Whitby', type: 'Townhouse', price: 675000, date: '2024-03-13', days: 15 },
        ];
        this.populateSalesTable(demoSales);
    }

    updateStats(data) {
        if (data.avg_price) {
            const el = document.getElementById('avg-price');
            if (el) el.textContent = '$' + data.avg_price.toLocaleString();
        }
        if (data.active_listings) {
            const el = document.getElementById('active-listings');
            if (el) el.textContent = data.active_listings.toLocaleString();
        }
        if (data.avg_days_on_market) {
            const el = document.getElementById('days-market');
            if (el) el.textContent = data.avg_days_on_market.toFixed(1);
        }
    }

    updateTrendsChart(trends) {
        if (trends && trends.length > 0 && this.charts.priceTrends) {
            this.charts.priceTrends.data.labels = trends.map(t => t.month);
            this.charts.priceTrends.data.datasets[0].data = trends.map(t => t.avg_price);
            this.charts.priceTrends.update();
        }
    }

    populateSalesTable(sales) {
        const tbody = document.getElementById('sales-table-body');
        if (!tbody) return;
        tbody.innerHTML = sales.map(sale => `
            <tr>
                <td>${sale.address}</td>
                <td>${sale.municipality}</td>
                <td>${sale.type}</td>
                <td>$${sale.price.toLocaleString()}</td>
                <td>${sale.date}</td>
                <td>${sale.days}</td>
            </tr>
        `).join('');
    }

    loadDemoData() {
        console.log('Loading demo data');
    }

    async fetchData() {
        console.log('fetchData called');
        this.showLoading();

        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'fetch', parameters: { source: 'zolo', limit: 50 } })
            });
            const result = await response.json();
            console.log('Fetch result:', result);

            if (result.success) {
                this.showToast('Data fetched successfully', 'success');
                await this.loadDashboardData();
            } else {
                this.showToast('Failed: ' + (result.error || 'Unknown error'), 'error');
            }
        } catch (error) {
            console.error('Fetch error:', error);
            this.showToast('Error: ' + error.message, 'error');
        } finally {
            this.hideLoading();
        }
    }

    async exportReport() {
        console.log('exportReport called');
        try {
            const response = await fetch('/api/realestate/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command: 'report', parameters: { output: 'durham_realestate_report.md' } })
            });
            const result = await response.json();
            this.showToast(result.success ? 'Report generated' : 'Report failed', result.success ? 'success' : 'error');
        } catch (error) {
            console.error('Export error:', error);
            this.showToast('Export error', 'error');
        }
    }

    updatePriceChart(period) {
        console.log('Chart period:', period);
    }

    filterTable(searchTerm) {
        document.querySelectorAll('#sales-table-body tr').forEach(row => {
            row.style.display = row.textContent.toLowerCase().includes(searchTerm.toLowerCase()) ? '' : 'none';
        });
    }

    filterByPropertyType(type) {
        document.querySelectorAll('#sales-table-body tr').forEach(row => {
            if (!type) {
                row.style.display = '';
            } else {
                const rowType = row.cells[2]?.textContent?.toLowerCase() || '';
                row.style.display = rowType.includes(type.toLowerCase()) ? '' : 'none';
            }
        });
    }

    showLoading() {
        const overlay = document.getElementById('loading-overlay');
        if (overlay) overlay.classList.add('visible');
    }

    hideLoading() {
        const overlay = document.getElementById('loading-overlay');
        if (overlay) overlay.classList.remove('visible');
    }

    showToast(message, type = 'info') {
        console.log(`Toast [${type}]: ${message}`);
        const container = document.getElementById('toast-container');
        if (!container) return;
        
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.innerHTML = `<span>${message}</span><button onclick="this.parentElement.remove()">x</button>`;
        container.appendChild(toast);
        setTimeout(() => toast.remove(), 5000);
    }
}

// Initialize
console.log('dashboard.js loaded');
window.dashboard = new MarketDashboard();

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        console.log('DOMContentLoaded - initializing dashboard');
        window.dashboard.init();
    });
} else {
    console.log('DOM ready - initializing dashboard');
    window.dashboard.init();
}
