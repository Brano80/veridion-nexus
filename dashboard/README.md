# Veridion Nexus Dashboard

Modern, comprehensive compliance dashboard for GDPR, eIDAS, and EU AI Act compliance management.

## Features

- **Real-time Updates**: Automatic data refresh every 10 seconds
- **Comprehensive Views**: All compliance features accessible through intuitive UI
- **Interactive Charts**: Visual analytics using Recharts
- **Responsive Design**: Works on desktop and mobile devices
- **Dark Theme**: Professional dark mode interface

## Pages

1. **Dashboard** (`/`) - Overview with key metrics and recent activity
2. **Compliance Records** (`/compliance`) - All compliance actions and eIDAS seals
3. **Data Subjects** (`/data-subjects`) - GDPR data subject rights management
4. **Human Oversight** (`/human-oversight`) - Review and approve/reject AI actions
5. **Risk Assessment** (`/risk-assessment`) - Risk analysis and visualization
6. **Data Breaches** (`/data-breaches`) - Breach reporting and management
7. **Consent Management** (`/consent`) - Consent tracking
8. **DPIA Tracking** (`/dpia`) - Data Protection Impact Assessments
9. **Retention Policies** (`/retention`) - Automated retention management
10. **Post-Market Monitoring** (`/monitoring`) - AI system monitoring
11. **AI-BOM** (`/ai-bom`) - CycloneDX AI-BOM export
12. **Green AI** (`/green-ai`) - Energy and carbon footprint tracking
13. **Webhooks** (`/webhooks`) - Webhook endpoint management
14. **Settings** (`/settings`) - System configuration

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm or yarn
- Backend API running on `http://127.0.0.1:8080`

### Installation

```bash
cd dashboard
npm install --legacy-peer-deps
```

### Development

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### Build

```bash
npm run build
npm start
```

## Technology Stack

- **Next.js 16** - React framework with App Router
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling
- **React Query** - Data fetching and caching
- **Recharts** - Chart visualization
- **Lucide React** - Icons
- **date-fns** - Date formatting

## API Integration

The dashboard connects to the Veridion Nexus API at `http://127.0.0.1:8080/api/v1`.

All API endpoints are documented in the Swagger UI at `http://127.0.0.1:8080/swagger-ui/`.

## Features

### Real-time Updates
- Automatic refresh every 10 seconds
- React Query for efficient data fetching
- Optimistic updates for better UX

### Responsive Design
- Mobile-friendly sidebar navigation
- Adaptive layouts for all screen sizes
- Touch-friendly interactions

### Data Visualization
- Risk distribution charts
- Timeline visualizations
- Energy consumption graphs
- Event tracking charts

## Configuration

Update the API base URL in individual page components if needed:

```typescript
const API_BASE = "http://127.0.0.1:8080/api/v1";
```

## License

Proprietary - Veridion Nexus
