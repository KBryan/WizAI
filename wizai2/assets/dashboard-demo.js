// Market Analysis Dashboard - POC Version for Real Estate Agents
// Professional dashboard with realistic demo data

class MarketDashboard {
    constructor() {
        this.charts = {};
        this.currentRegion = 'durham';
        this.demoMode = true;
        this.data = {
            stats: {},
            trends: [],
            municipalities: [],
            recentSales: [],
            comparisons: []
        };
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.initializeCharts();
        this.loadDemoData();
        this.animateStats();
    }

    setupEventListeners() {
        // Fetch data button
        document.getElementById('fetch-data-btn')?.addEventListener('click', () => {
            this.showLoading();
            setTimeout(() => {
                this.loadDemoData();
                this.hideLoading();
                this.showToast('Data refreshed successfully', 'success');
            }, 1500);
        });

        // Export button
        document.getElementById('export-btn')?.addEventListener('click', () => {
            this.generateClientReport();
        });

        // CMA button
        document.getElementById('cma-btn')?.addEventListener('click', () => {
            this.openCMAModal();
        });

        // Region selector
        document.getElementById('region-selector')?.addEventListener('change', (e) => {
            this.currentRegion = e.target.value;
            this.loadDemoData();
        });

        // Chart period buttons
        document.querySelectorAll('.chart-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.chart-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                this.updatePriceChart(e.target.dataset.period);
            });
        });

        // Table search
        document.getElementById('table-search')?.addEventListener('input', (e) => {
            this.filterTable(e.target.value);
        });

        // Property filter
        document.getElementById('property-filter')?.addEventListener('change', (e) => {
            this.filterByPropertyType(e.target.value);
        });
    }

    // Realistic Durham Region Demo Data
    getDemoData() {
        return {
            stats: {
                avg_price: 875000,
                median_price: 849000,
                active_listings: 847,
                sold_last_30: 156,
                avg_days_on_market: 11.3,
                price_per_sqft: 568,
                trend: '+3.8%'
            },
            trends: [
                { month: 'Oct 2024', avg_price: 845000, detached: 925000, condo: 625000 },
                { month: 'Nov 2024', avg_price: 852000, detached: 935000, condo: 635000 },
                { month: 'Dec 2024', avg_price: 861000, detached: 945000, condo: 645000 },
                { month: 'Jan 2025', avg_price: 868000, detached: 955000, condo: 655000 },
                { month: 'Feb 2025', avg_price: 872000, detached: 960000, condo: 660000 },
                { month: 'Mar 2025', avg_price: 875000, detached: 965000, condo: 665000 }
            ],
            municipalities: [
                { name: 'Ajax', avg_price: 945000, median_price: 925000, count: 142, trend: '+4.2%' },
                { name: 'Pickering', avg_price: 925000, median_price: 899000, count: 128, trend: '+3.8%' },
                { name: 'Whitby', avg_price: 965000, median_price: 945000, count: 156, trend: '+5.1%' },
                { name: 'Oshawa', avg_price: 785000, median_price: 765000, count: 267, trend: '+2.9%' },
                { name: 'Brooklin', avg_price: 1125000, median_price: 1095000, count: 89, trend: '+6.2%' }
            ],
            recentSales: [
                { address: '1455 Salem Rd N', municipality: 'Ajax', type: 'Detached', beds: 4, baths: 3, price: 985000, date: '2025-03-15', days: 7, sqft: 2100 },
                { address: '1234 Bayly St', municipality: 'Pickering', type: 'Semi', beds: 3, baths: 2, price: 875000, date: '2025-03-14', days: 12, sqft: 1650 },
                { address: '789 Simcoe St S', municipality: 'Oshawa', type: 'Detached', beds: 3, baths: 2, price: 765000, date: '2025-03-13', days: 9, sqft: 1450 },
                { address: '456 Taunton Rd E', municipality: 'Whitby', type: 'Townhouse', beds: 3, baths: 3, price: 725000, date: '2025-03-12', days: 15, sqft: 1600 },
                { address: '901 Dundas St W', municipality: 'Whitby', type: 'Detached', beds: 4, baths: 4, price: 1125000, date: '2025-03-11', days: 5, sqft: 2800 },
                { address: '234 Brock St N', municipality: 'Ajax', type: 'Condo', beds: 2, baths: 2, price: 625000, date: '2025-03-10', days: 22, sqft: 980 },
                { address: '567 Cochrane St', municipality: 'Pickering', type: 'Detached', beds: 4, baths: 3, price: 945000, date: '2025-03-09', days: 8, sqft: 1950 },
                { address: '890 Dunning Ave', municipality: 'Oshawa', type: 'Semi', beds: 3, baths: 2, price: 695000, date: '2025-03-08', days: 18, sqft: 1350 }
            ]
        };
    }

    loadDemoData() {
        const data = this.getDemoData();
        
        // Update stats with animation
        this.animateValue('avg-price', data.stats.avg_price, '$');
        this.animateValue('active-listings', data.stats.active_listings, '');
        this.animateValue('days-market', data.stats.avg_days_on_market, '', 1);
        this.animateValue('price-sqft', data.stats.price_per_sqft, '$');
        
        // Update trends
        this.data.trends = data.trends;
        this.updateCharts(data);
        
        // Update table
        this.populateSalesTable(data.recentSales);
        
        // Update comparisons
        this.updateMunicipalityComparison(data.municipalities);
        
        // Show last updated
        document.getElementById('last-updated').textContent = new Date().toLocaleString();
    }

    initializeCharts() {
        // Price Trends Chart
        const priceCtx = document.getElementById('price-trends-chart');
        if (priceCtx) {
            this.charts.priceTrends = new Chart(priceCtx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Average Price',
                        data: [],
                        borderColor: '#6366f1',
                        backgroundColor: 'rgba(99, 102, 241, 0.1)',
                        borderWidth: 3,
                        fill: true,
                        tension: 0.4,
                        pointRadius: 4,
                        pointBackgroundColor: '#6366f1'
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: {
                        mode: 'index',
                        intersect: false
                    },
                    plugins: {
                        legend: { display: false },
                        tooltip: {
                            callbacks: {
                                label: function(context) {
                                    return 'Avg: $' + context.parsed.y.toLocaleString();
                                }
                            }
                        }
                    },
                    scales: {
                        y: {
                            beginAtZero: false,
                            ticks: {
                                callback: function(value) {
                                    return '$' + (value / 1000) + 'k';
                                },
                                color: '#6b7280'
                            },
                            grid: { color: '#2a2a35' }
                        },
                        x: {
                            ticks: { color: '#6b7280' },
                            grid: { display: false }
                        }
                    }
                }
            });
        }

        // Municipality Chart
        const munCtx = document.getElementById('municipality-chart');
        if (munCtx) {
            this.charts.municipality = new Chart(munCtx, {
                type: 'bar',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Avg Price',
                        data: [],
                        backgroundColor: [
                            '#6366f1', '#8b5cf6', '#ec4899', '#14b8a6', '#f59e0b'
                        ],
                        borderRadius: 6
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { display: false },
                        tooltip: {
                            callbacks: {
                                label: function(context) {
                                    return '$' + context.parsed.y.toLocaleString();
                                }
                            }
                        }
                    },
                    scales: {
                        y: {
                            ticks: {
                                callback: function(value) {
                                    return '$' + (value / 1000) + 'k';
                                },
                                color: '#6b7280'
                            },
                            grid: { color: '#2a2a35' }
                        },
                        x: {
                            ticks: { color: '#6b7280' },
                            grid: { display: false }
                        }
                    }
                }
            });
        }

        // Property Type Chart
        const propCtx = document.getElementById('property-type-chart');
        if (propCtx) {
            this.charts.propertyType = new Chart(propCtx, {
                type: 'doughnut',
                data: {
                    labels: ['Detached', 'Semi', 'Townhouse', 'Condo'],
                    datasets: [{
                        data: [42, 18, 28, 12],
                        backgroundColor: ['#6366f1', '#8b5cf6', '#ec4899', '#14b8a6'],
                        borderWidth: 0
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: {
                            position: 'bottom',
                            labels: {
                                color: '#a0a0b0',
                                padding: 20,
                                usePointStyle: true
                            }
                        }
                    }
                }
            });
        }
    }

    updateCharts(data) {
        // Update price trends
        if (this.charts.priceTrends) {
            this.charts.priceTrends.data.labels = data.trends.map(t => t.month);
            this.charts.priceTrends.data.datasets[0].data = data.trends.map(t => t.avg_price);
            this.charts.priceTrends.update('active');
        }

        // Update municipality chart
        if (this.charts.municipality) {
            this.charts.municipality.data.labels = data.municipalities.map(m => m.name);
            this.charts.municipality.data.datasets[0].data = data.municipalities.map(m => m.avg_price);
            this.charts.municipality.update('active');
        }
    }

    populateSalesTable(sales) {
        const tbody = document.getElementById('sales-table-body');
        if (!tbody) return;
        
        tbody.innerHTML = sales.map(sale => `
            <tr>
                <td><strong>${sale.address}</strong></td>
                <td><span class="badge">${sale.municipality}</span></td>
                <td>${sale.type}</td>
                <td><strong class="price">$${sale.price.toLocaleString()}</strong></td>
                <td>${new Date(sale.date).toLocaleDateString()}</td>
                <td><span class="days ${sale.days < 10 ? 'fast' : sale.days > 20 ? 'slow' : ''}">${sale.days} days</span></td>
            </tr>
        `).join('');
    }

    updateMunicipalityComparison(municipalities) {
        const container = document.getElementById('municipality-comparison');
        if (!container) return;
        
        container.innerHTML = municipalities.map(m => `
            <div class="comparison-row">
                <div class="mun-name">${m.name}</div>
                <div class="mun-stats">
                    <div class="stat">
                        <span class="label">Average</span>
                        <span class="value">$${m.avg_price.toLocaleString()}</span>
                    </div>
                    <div class="stat">
                        <span class="label">Listings</span>
                        <span class="value">${m.count}</span>
                    </div>
                    <div class="trend ${m.trend.startsWith('+') ? 'up' : 'down'}">${m.trend}</div>
                </div>
            </div>
        `).join('');
    }

    animateValue(elementId, value, prefix = '', decimals = 0) {
        const element = document.getElementById(elementId);
        if (!element) return;
        
        const duration = 1500;
        const start = 0;
        const startTime = performance.now();
        
        const animate = (currentTime) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            
            // Easing function
            const easeOut = 1 - Math.pow(1 - progress, 3);
            const current = start + (value - start) * easeOut;
            
            if (decimals > 0) {
                element.textContent = prefix + current.toFixed(decimals);
            } else {
                element.textContent = prefix + Math.round(current).toLocaleString();
            }
            
            if (progress < 1) {
                requestAnimationFrame(animate);
            }
        };
        
        requestAnimationFrame(animate);
    }

    animateStats() {
        const stats = document.querySelectorAll('.stat-card');
        stats.forEach((stat, index) => {
            stat.style.opacity = '0';
            stat.style.transform = 'translateY(20px)';
            setTimeout(() => {
                stat.style.transition = 'all 0.6s ease';
                stat.style.opacity = '1';
                stat.style.transform = 'translateY(0)';
            }, index * 100);
        });
    }

    openCMAModal() {
        const modal = document.getElementById('cma-modal');
        if (modal) {
            modal.classList.add('visible');
            // Populate with sample CMA data
            this.generateCMAResults();
        }
    }

    generateCMAResults() {
        const results = document.getElementById('cma-results');
        if (!results) return;
        
        const sampleComparables = [
            { address: '1450 Salem Rd N', price: 975000, beds: 4, baths: 3, sqft: 2150, distance: 0.2 },
            { address: '1460 Salem Rd N', price: 965000, beds: 4, baths: 3, sqft: 2100, distance: 0.3 },
            { address: '1445 Salem Rd N', price: 995000, beds: 4, baths: 4, sqft: 2250, distance: 0.4 },
        ];
        
        const avgPrice = sampleComparables.reduce((a, b) => a + b.price, 0) / sampleComparables.length;
        
        results.innerHTML = `
            <div class="cma-summary">
                <h4>Comparative Market Analysis</h4>
                <div class="cma-price">$${Math.round(avgPrice).toLocaleString()}</div>
                <div class="cma-label">Estimated Market Value</div>
                <div class="cma-range">Range: $945,000 - $995,000</div>
            </div>
            <div class="cma-comparables">
                <h5>Comparable Sales (Last 90 Days)</h5>
                ${sampleComparables.map(c => `
                    <div class="comparable">
                        <div class="comp-address">${c.address}</div>
                        <div class="comp-details">
                            <span>${c.beds} bed, ${c.baths} bath</span>
                            <span>${c.sqft.toLocaleString()} sqft</span>
                            <span class="comp-price">$${c.price.toLocaleString()}</span>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    }

    generateClientReport() {
        const report = `
# Durham Region Real Estate Market Report
**Prepared for:** Client Name  
**Date:** ${new Date().toLocaleDateString()}  
**Agent:** [Your Name]

## Executive Summary
The Durham Region real estate market continues to show strong performance with average prices up 3.8% over the past 6 months. Current inventory remains tight with only 847 active listings across all municipalities.

## Key Statistics
- **Average Price:** $875,000
- **Median Price:** $849,000
- **Average Days on Market:** 11.3 days
- **Price per SqFt:** $568

## Market Trends
Prices have steadily increased from $845,000 in October 2024 to $875,000 in March 2025, representing a healthy 3.5% growth rate.

## Municipality Breakdown
1. **Brooklin** - $1,125,000 avg (+6.2%)
2. **Whitby** - $965,000 avg (+5.1%)
3. **Ajax** - $945,000 avg (+4.2%)
4. **Pickering** - $925,000 avg (+3.8%)
5. **Oshawa** - $785,000 avg (+2.9%)

## Recommendations
- Sellers: Market conditions favor sellers with low inventory
- Buyers: Act quickly on well-priced properties
- Investors: Oshawa offers best value proposition

---
*Report generated by OpenSpec Market Analysis Dashboard*
        `;
        
        const blob = new Blob([report], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `Durham_Market_Report_${new Date().toISOString().split('T')[0]}.md`;
        a.click();
        
        this.showToast('Client report generated!', 'success');
    }

    filterTable(searchTerm) {
        const rows = document.querySelectorAll('#sales-table-body tr');
        rows.forEach(row => {
            const text = row.textContent.toLowerCase();
            row.style.display = text.includes(searchTerm.toLowerCase()) ? '' : 'none';
        });
    }

    filterByPropertyType(type) {
        const rows = document.querySelectorAll('#sales-table-body tr');
        rows.forEach(row => {
            if (!type) {
                row.style.display = '';
            } else {
                const rowType = row.cells[2]?.textContent?.toLowerCase() || '';
                row.style.display = rowType.includes(type.toLowerCase()) ? '' : 'none';
            }
        });
    }

    updatePriceChart(period) {
        console.log('Updating chart for period:', period);
        // Would fetch different time period data
    }

    showLoading() {
        document.getElementById('loading-overlay')?.classList.add('visible');
    }

    hideLoading() {
        document.getElementById('loading-overlay')?.classList.remove('visible');
    }

    showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        if (!container) return;
        
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.innerHTML = `
            <span>${message}</span>
            <button onclick="this.parentElement.remove()">×</button>
        `;
        container.appendChild(toast);
        
        setTimeout(() => toast.remove(), 4000);
    }
}

// Initialize dashboard
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new MarketDashboard();
});
