'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import WizardLayout from '../components/WizardLayout';
// Wizard endpoints work without authentication (optional auth)
import { Sparkles, ArrowRight, ArrowLeft, CheckCircle2, Loader2 } from 'lucide-react';

interface CompanyProfile {
  company_name: string;
  industry: string;
  company_size: string;
  country: string;
  regulatory_requirements: string[];
  ai_use_cases: string[];
  deployment_preference: string;
  estimated_ai_systems: number;
}

interface RecommendedModule {
  module_name: string;
  display_name: string;
  description: string | null;
  category: string;
  recommendation_reason: string;
  priority: string;
  requires_license: boolean;
}

interface ModuleRecommendation {
  core_modules: RecommendedModule[];
  recommended_modules: RecommendedModule[];
  required_count: number;
  recommended_count: number;
  optional_count: number;
}

interface PricingBreakdown {
  base_price: number;
  per_system_price: number;
  module_prices: Record<string, number>;
  total_monthly: number;
  total_annual: number;
  savings_annual: number;
}

interface Subscription {
  id: string;
  company_id: string;
  subscription_type: string;
  status: string;
  trial_start_date: string | null;
  trial_end_date: string | null;
  days_remaining: number | null;
  monthly_price: number | null;
  annual_price: number | null;
}

const INDUSTRIES = [
  { value: 'FINANCIAL_SERVICES', label: 'Financial Services' },
  { value: 'HEALTHCARE', label: 'Healthcare' },
  { value: 'INSURANCE', label: 'Insurance' },
  { value: 'E_COMMERCE', label: 'E-Commerce' },
  { value: 'SAAS', label: 'SaaS' },
  { value: 'STARTUP', label: 'Startup' },
  { value: 'SME', label: 'SME' },
  { value: 'GOVERNMENT', label: 'Government' },
  { value: 'OTHER', label: 'Other' },
];

const COMPANY_SIZES = [
  { value: 'STARTUP', label: 'Startup (1-10 employees)' },
  { value: 'SME', label: 'SME (11-50 employees)' },
  { value: 'MID_MARKET', label: 'Mid-Market (51-200 employees)' },
  { value: 'ENTERPRISE', label: 'Enterprise (200+ employees)' },
];

const REGULATORY_REQUIREMENTS = [
  { value: 'GDPR', label: 'GDPR' },
  { value: 'EU_AI_ACT', label: 'EU AI Act' },
  { value: 'DORA', label: 'DORA' },
  { value: 'NIS2', label: 'NIS2' },
  { value: 'MIFID_II', label: 'MiFID II' },
  { value: 'SOLVENCY_II', label: 'Solvency II' },
];

const AI_USE_CASES = [
  { value: 'CREDIT_SCORING', label: 'Credit Scoring' },
  { value: 'FRAUD_DETECTION', label: 'Fraud Detection' },
  { value: 'CUSTOMER_SERVICE', label: 'Customer Service' },
  { value: 'CONTENT_GENERATION', label: 'Content Generation' },
  { value: 'RECOMMENDATION_ENGINE', label: 'Recommendation Engine' },
  { value: 'MEDICAL_DIAGNOSIS', label: 'Medical Diagnosis' },
];

const DEPLOYMENT_OPTIONS = [
  { value: 'SDK', label: 'SDK Mode (Easiest - 15 minutes)' },
  { value: 'PROXY', label: 'Proxy Mode (1-2 hours)' },
  { value: 'GATEWAY', label: 'Gateway Mode (Enterprise)' },
];

const EU_COUNTRIES = [
  { value: 'AT', label: 'Austria' },
  { value: 'BE', label: 'Belgium' },
  { value: 'BG', label: 'Bulgaria' },
  { value: 'HR', label: 'Croatia' },
  { value: 'CY', label: 'Cyprus' },
  { value: 'CZ', label: 'Czech Republic' },
  { value: 'DK', label: 'Denmark' },
  { value: 'EE', label: 'Estonia' },
  { value: 'FI', label: 'Finland' },
  { value: 'FR', label: 'France' },
  { value: 'DE', label: 'Germany' },
  { value: 'GR', label: 'Greece' },
  { value: 'HU', label: 'Hungary' },
  { value: 'IE', label: 'Ireland' },
  { value: 'IT', label: 'Italy' },
  { value: 'LV', label: 'Latvia' },
  { value: 'LT', label: 'Lithuania' },
  { value: 'LU', label: 'Luxembourg' },
  { value: 'MT', label: 'Malta' },
  { value: 'NL', label: 'Netherlands' },
  { value: 'PL', label: 'Poland' },
  { value: 'PT', label: 'Portugal' },
  { value: 'RO', label: 'Romania' },
  { value: 'SK', label: 'Slovakia' },
  { value: 'SI', label: 'Slovenia' },
  { value: 'ES', label: 'Spain' },
  { value: 'SE', label: 'Sweden' },
];

export default function WizardPage() {
  const router = useRouter();
  const [step, setStep] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const [profile, setProfile] = useState<Partial<CompanyProfile>>({
    company_name: '',
    industry: '',
    company_size: '',
    country: '',
    regulatory_requirements: [],
    ai_use_cases: [],
    deployment_preference: 'SDK',
    estimated_ai_systems: 1,
  });

  const [recommendations, setRecommendations] = useState<ModuleRecommendation | null>(null);
  const [selectedModules, setSelectedModules] = useState<string[]>([]);
  const [pricing, setPricing] = useState<PricingBreakdown | null>(null);
  const [companyId, setCompanyId] = useState<string | null>(null);
  const [subscription, setSubscription] = useState<Subscription | null>(null);
  const [agreedToTerms, setAgreedToTerms] = useState(false);

  const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8080/api/v1';

  const handleNext = async () => {
    setError(null);

    if (step === 1) {
      // Validate step 1
      if (!profile.company_name || !profile.industry || !profile.company_size || !profile.country) {
        setError('Please fill in all required fields');
        return;
      }
      setStep(2);
    } else if (step === 2) {
      // Validate step 2
      if (profile.regulatory_requirements?.length === 0 && profile.ai_use_cases?.length === 0) {
        setError('Please select at least one regulatory requirement or AI use case');
        return;
      }
      setStep(3);
      // Fetch recommendations and pricing when moving to step 3
      await fetchRecommendationsAndPricing();
    } else if (step === 3) {
      // Validate terms agreement
      if (!agreedToTerms) {
        setError('Please agree to the Terms of Service to start your trial');
        return;
      }
      // Start trial
      await startTrial();
    }
  };

  const fetchRecommendationsAndPricing = async () => {
    setLoading(true);
    setError(null);

    try {
      // Get module recommendations (wizard endpoints work without auth)
      const recResponse = await fetch(`${API_BASE}/wizard/recommend-modules`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          industry: profile.industry,
          regulatory_requirements: profile.regulatory_requirements || [],
          ai_use_cases: profile.ai_use_cases || [],
        }),
      });

      if (!recResponse.ok) {
        const errorData = await recResponse.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to get recommendations');
      }

      const recData = await recResponse.json();
      setRecommendations(recData);
      
      // Auto-select core modules (always included, no price)
      const coreModuleNames = (recData.core_modules || [])
        .map((m: RecommendedModule) => m.module_name);
      
      // Auto-select required modules
      const required = recData.recommended_modules
        .filter((m: RecommendedModule) => m.priority === 'REQUIRED')
        .map((m: RecommendedModule) => m.module_name);
      
      // Combine core and required modules
      setSelectedModules([...coreModuleNames, ...required]);

      // Calculate pricing with only recommended modules (exclude core modules)
      await calculatePricing(required);
    } catch (err: any) {
      let errorMessage = 'An error occurred';
      
      if (err instanceof TypeError && err.message.includes('fetch')) {
        errorMessage = 'Cannot connect to backend API. Please make sure the backend is running on http://127.0.0.1:8080';
      } else if (err instanceof Error) {
        errorMessage = err.message;
      }
      
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const calculatePricing = async (modules: string[] = selectedModules) => {
    try {
      const response = await fetch(`${API_BASE}/wizard/calculate-price`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          selected_modules: modules,
          num_systems: profile.estimated_ai_systems || 1,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to calculate price');
      }

      const data = await response.json();
      setPricing(data);
    } catch (err: any) {
      console.error('Error calculating pricing:', err);
      // Set fallback pricing if calculation fails
      const fallbackPricing: PricingBreakdown = {
        base_price: 299.00,
        per_system_price: 100.00,
        module_prices: {},
        total_monthly: 299.00 + (100.00 * (profile.estimated_ai_systems || 1)),
        total_annual: (299.00 + (100.00 * (profile.estimated_ai_systems || 1))) * 12 * 0.85,
        savings_annual: (299.00 + (100.00 * (profile.estimated_ai_systems || 1))) * 12 * 0.15,
      };
      setPricing(fallbackPricing);
    }
  };

  const startTrial = async () => {
    setLoading(true);
    setError(null);

    try {
      // Create company profile
      const profileResponse = await fetch(`${API_BASE}/wizard/company-profile`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(profile),
      });

      if (!profileResponse.ok) {
        const errorData = await profileResponse.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to create company profile');
      }

      const profileData = await profileResponse.json();
      setCompanyId(profileData.id);

      // Start trial
      const trialResponse = await fetch(`${API_BASE}/wizard/start-trial`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          company_id: profileData.id,
          selected_modules: selectedModules,
          estimated_ai_systems: profile.estimated_ai_systems || 1,
        }),
      });

      if (!trialResponse.ok) {
        const errorData = await trialResponse.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to start trial');
      }

      const trialData = await trialResponse.json();
      setSubscription(trialData);
      
      // Redirect to dashboard after successful trial start
      setTimeout(() => {
        router.push('/dashboard');
      }, 2000);
    } catch (err: any) {
      let errorMessage = 'An error occurred';
      
      if (err instanceof TypeError && err.message.includes('fetch')) {
        errorMessage = 'Cannot connect to backend API. Please make sure the backend is running on http://127.0.0.1:8080';
      } else if (err instanceof Error) {
        errorMessage = err.message;
      }
      
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleBack = () => {
    if (step > 1) {
      setStep(step - 1);
      setError(null);
    }
  };

  const toggleModule = async (moduleName: string) => {
    // Get core module names (cannot be toggled)
    const coreModuleNames = recommendations?.core_modules?.map((m: RecommendedModule) => m.module_name) || [];
    
    // Prevent toggling core modules
    if (coreModuleNames.includes(moduleName)) {
      return;
    }
    
    const newSelection = selectedModules.includes(moduleName)
      ? selectedModules.filter(m => m !== moduleName)
      : [...selectedModules, moduleName];
    
    setSelectedModules(newSelection);
    
    // Recalculate pricing when modules change (exclude core modules from pricing)
    if (pricing) {
      const modulesForPricing = newSelection.filter(m => !coreModuleNames.includes(m));
      await calculatePricing(modulesForPricing);
    }
  };

  return (
    <WizardLayout>
      <div className="max-w-5xl mx-auto px-6 py-8 flex flex-col min-h-[calc(100vh-80px)]">
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-1">
            <Sparkles className="w-6 h-6 text-emerald-400" />
            <h1 className="text-2xl font-bold text-slate-100">Setup Wizard</h1>
          </div>
          <p className="text-sm text-slate-400">Get started in minutes with automated compliance</p>
        </div>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between mb-1">
            {[1, 2, 3].map((s) => (
              <div
                key={s}
                className={`flex-1 h-1.5 mx-1 rounded transition-all duration-300 ${
                  step >= s
                    ? 'bg-emerald-500'
                    : 'bg-slate-700'
                }`}
              />
            ))}
          </div>
          <div className="text-xs text-slate-400 text-center">
            Step {step} of 3
          </div>
        </div>

        {error && (
          <div className="mb-3 p-2 text-sm bg-red-900/20 border border-red-800 rounded-lg text-red-400">
            {error}
          </div>
        )}

        <div className="flex-1 overflow-y-auto">
          {/* Step 1: Company Info */}
          {step === 1 && (
            <div className="bg-slate-900 border border-slate-800 rounded-lg p-6 space-y-4 transition-opacity duration-300">
              <h2 className="text-xl font-semibold text-slate-100">Company Information</h2>
            
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">
                  Company Name *
                </label>
                <input
                  type="text"
                  value={profile.company_name}
                  onChange={(e) => setProfile({ ...profile, company_name: e.target.value })}
                  className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                  placeholder="Acme Corp"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">
                  Industry *
                </label>
                <select
                  value={profile.industry}
                  onChange={(e) => setProfile({ ...profile, industry: e.target.value })}
                  className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                >
                  <option value="">Select industry</option>
                  {INDUSTRIES.map(industry => (
                    <option key={industry.value} value={industry.value}>
                      {industry.label}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">
                  Company Size *
                </label>
                <select
                  value={profile.company_size}
                  onChange={(e) => setProfile({ ...profile, company_size: e.target.value })}
                  className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                >
                  <option value="">Select size</option>
                  {COMPANY_SIZES.map(size => (
                    <option key={size.value} value={size.value}>
                      {size.label}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">
                  Country *
                </label>
                <select
                  value={profile.country}
                  onChange={(e) => setProfile({ ...profile, country: e.target.value })}
                  className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                >
                  <option value="">Select country</option>
                  {EU_COUNTRIES.map(country => (
                    <option key={country.value} value={country.value}>
                      {country.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          )}

          {/* Step 2: AI Usage */}
          {step === 2 && (
            <div className="bg-slate-900 border border-slate-800 rounded-lg p-6 space-y-4 transition-opacity duration-300">
              <h2 className="text-xl font-semibold text-slate-100">AI Usage & Requirements</h2>
              
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-2">
                  Which regulations apply to your business? *
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {REGULATORY_REQUIREMENTS.map(req => (
                    <label
                      key={req.value}
                      className={`flex items-center p-2 text-xs border rounded-lg cursor-pointer transition-all ${
                        profile.regulatory_requirements?.includes(req.value)
                          ? 'bg-emerald-900/20 border-emerald-700 text-emerald-400'
                          : 'bg-slate-800 border-slate-700 text-slate-300 hover:bg-slate-750 hover:border-slate-600'
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={profile.regulatory_requirements?.includes(req.value) || false}
                        onChange={(e) => {
                          const current = profile.regulatory_requirements || [];
                          setProfile({
                            ...profile,
                            regulatory_requirements: e.target.checked
                              ? [...current, req.value]
                              : current.filter(r => r !== req.value)
                          });
                        }}
                        className="mr-1.5 w-3.5 h-3.5 text-emerald-600 bg-slate-700 border-slate-600 rounded focus:ring-emerald-500"
                      />
                      <span className="text-xs">{req.label}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-2">
                  AI Use Cases
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {AI_USE_CASES.map(useCase => (
                    <label
                      key={useCase.value}
                      className={`flex items-center p-2 text-xs border rounded-lg cursor-pointer transition-all ${
                        profile.ai_use_cases?.includes(useCase.value)
                          ? 'bg-emerald-900/20 border-emerald-700 text-emerald-400'
                          : 'bg-slate-800 border-slate-700 text-slate-300 hover:bg-slate-750 hover:border-slate-600'
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={profile.ai_use_cases?.includes(useCase.value) || false}
                        onChange={(e) => {
                          const current = profile.ai_use_cases || [];
                          setProfile({
                            ...profile,
                            ai_use_cases: e.target.checked
                              ? [...current, useCase.value]
                              : current.filter(u => u !== useCase.value)
                          });
                        }}
                        className="mr-1.5 w-3.5 h-3.5 text-emerald-600 bg-slate-700 border-slate-600 rounded focus:ring-emerald-500"
                      />
                      <span className="text-xs">{useCase.label}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-slate-300 mb-1">
                    Estimated AI Systems
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={profile.estimated_ai_systems}
                    onChange={(e) => setProfile({ ...profile, estimated_ai_systems: parseInt(e.target.value) || 1 })}
                    className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                  />
                </div>

                <div>
                  <label className="block text-xs font-medium text-slate-300 mb-1">
                    Deployment Preference
                  </label>
                  <select
                    value={profile.deployment_preference}
                    onChange={(e) => setProfile({ ...profile, deployment_preference: e.target.value })}
                    className="w-full px-3 py-1.5 text-sm bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-all"
                  >
                    {DEPLOYMENT_OPTIONS.map(opt => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>
          )}

          {/* Step 3: Modules & Pricing */}
          {step === 3 && (
            <div className="space-y-4 transition-opacity duration-300">
              {loading && !recommendations ? (
                <div className="bg-slate-900 border border-slate-800 rounded-lg p-6 text-center">
                  <Loader2 className="w-6 h-6 text-emerald-400 animate-spin mx-auto mb-2" />
                  <p className="text-sm text-slate-400">Loading recommendations...</p>
                </div>
              ) : recommendations ? (
                <>
                  {/* Core Modules - Always included, no pricing */}
                  {recommendations.core_modules && recommendations.core_modules.length > 0 && (
                    <div className="bg-slate-900 border border-slate-800 rounded-lg p-4 mb-4">
                      <h2 className="text-lg font-semibold text-slate-100 mb-2">
                        Core Modules
                      </h2>
                      <p className="text-xs text-slate-400 mb-3">
                        Always included - no additional cost
                      </p>

                      <div className="space-y-2">
                        {recommendations.core_modules.map((module) => (
                          <div
                            key={module.module_name}
                            className="p-2 border border-slate-700 rounded-lg bg-slate-800/50 opacity-75"
                          >
                            <div className="flex items-center gap-2">
                              <input
                                type="checkbox"
                                checked={true}
                                disabled
                                className="w-3.5 h-3.5 text-emerald-600 bg-slate-700 border-slate-600 rounded cursor-not-allowed"
                              />
                              <h3 className="text-sm font-semibold text-slate-300 flex-1">{module.display_name}</h3>
                              <span className="text-xs px-1.5 py-0.5 rounded bg-blue-900/30 text-blue-400">
                                INCLUDED
                              </span>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Recommended Modules */}
                  <div className="bg-slate-900 border border-slate-800 rounded-lg p-4">
                    <h2 className="text-lg font-semibold text-slate-100 mb-2">
                      Recommended Modules
                    </h2>
                    <p className="text-xs text-slate-400 mb-3">
                      Based on your industry and requirements:
                    </p>

                    <div className="space-y-2 mb-4 max-h-48 overflow-y-auto">
                      {recommendations.recommended_modules.map((module) => (
                        <div
                          key={module.module_name}
                          className={`p-2 border rounded-lg cursor-pointer transition-all ${
                            selectedModules.includes(module.module_name)
                              ? 'border-emerald-500 bg-emerald-900/20'
                              : 'border-slate-700 hover:border-slate-600 bg-slate-800'
                          }`}
                          onClick={() => toggleModule(module.module_name)}
                        >
                          <div className="flex items-center gap-2">
                            <input
                              type="checkbox"
                              checked={selectedModules.includes(module.module_name)}
                              onChange={() => toggleModule(module.module_name)}
                              className="w-3.5 h-3.5 text-emerald-600 bg-slate-700 border-slate-600 rounded focus:ring-emerald-500"
                            />
                            <h3 className="text-sm font-semibold text-slate-100 flex-1">{module.display_name}</h3>
                            <span className={`text-xs px-1.5 py-0.5 rounded ${
                              module.priority === 'REQUIRED'
                                ? 'bg-red-900/30 text-red-400'
                                : module.priority === 'RECOMMENDED'
                                ? 'bg-yellow-900/30 text-yellow-400'
                                : 'bg-slate-700 text-slate-400'
                            }`}>
                              {module.priority}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>

                    <div className="bg-slate-800 p-2 rounded-lg">
                      <div className="grid grid-cols-3 gap-2 text-center">
                        <div>
                          <div className="text-lg font-bold text-red-400">
                            {recommendations.required_count}
                          </div>
                          <div className="text-xs text-slate-400">Required</div>
                        </div>
                        <div>
                          <div className="text-lg font-bold text-yellow-400">
                            {recommendations.recommended_count}
                          </div>
                          <div className="text-xs text-slate-400">Recommended</div>
                        </div>
                        <div>
                          <div className="text-lg font-bold text-slate-400">
                            {recommendations.optional_count}
                          </div>
                          <div className="text-xs text-slate-400">Optional</div>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Pricing Summary */}
                  {(pricing || selectedModules.length > 0) && (
                    <div className="bg-slate-900 border border-slate-800 rounded-lg p-4">
                      <h2 className="text-lg font-semibold text-slate-100 mb-3">Pricing Summary</h2>

                      <div className="bg-slate-800 p-3 rounded-lg space-y-2 mb-3 text-sm">
                        {pricing ? (
                          <>
                            <div className="flex justify-between text-slate-300">
                              <span>Base Platform</span>
                              <span className="font-semibold">€{pricing.base_price.toFixed(2)}/mo</span>
                            </div>
                            <div className="flex justify-between text-slate-300">
                              <span className="text-xs">
                                {profile.estimated_ai_systems} AI System(s) × €{pricing.per_system_price.toFixed(2)}
                              </span>
                              <span className="font-semibold text-xs">
                                €{(pricing.per_system_price * (profile.estimated_ai_systems || 1)).toFixed(2)}/mo
                              </span>
                            </div>
                            {selectedModules.length > 0 && (() => {
                              const coreModuleNames = recommendations?.core_modules?.map((m: RecommendedModule) => m.module_name) || [];
                              const modulesWithPrice = selectedModules
                                .filter(moduleName => !coreModuleNames.includes(moduleName))
                                .map(moduleName => {
                                  const price = pricing.module_prices[moduleName] || 0;
                                  return { moduleName, price };
                                })
                                .filter(m => m.price > 0);
                              
                              return modulesWithPrice.length > 0 ? (
                                <div className="border-t border-slate-700 pt-2">
                                  <div className="text-xs font-medium mb-1 text-slate-300">Modules:</div>
                                  <div className="max-h-20 overflow-y-auto space-y-1">
                                    {modulesWithPrice.map(({ moduleName, price }) => (
                                      <div key={moduleName} className="flex justify-between text-xs text-slate-400">
                                        <span className="truncate">{moduleName.replace(/_/g, ' ')}</span>
                                        <span>€{price.toFixed(2)}</span>
                                      </div>
                                    ))}
                                  </div>
                                </div>
                              ) : null;
                            })()}
                            <div className="border-t border-slate-700 pt-2 flex justify-between font-bold text-slate-100">
                              <span>Total (Monthly)</span>
                              <span className="text-emerald-400">€{pricing.total_monthly.toFixed(2)}/mo</span>
                            </div>
                            <div className="flex justify-between text-xs text-slate-400">
                              <span>Annual: €{pricing.total_annual.toFixed(2)}</span>
                              <span className="text-emerald-400">Save €{pricing.savings_annual.toFixed(2)}</span>
                            </div>
                          </>
                        ) : (
                          <div className="text-center py-4">
                            <p className="text-sm text-slate-400 mb-2">Calculating pricing...</p>
                            <p className="text-xs text-slate-500">
                              {selectedModules.length} module(s) selected
                            </p>
                          </div>
                        )}
                      </div>

                      <div className="bg-emerald-900/20 border border-emerald-800 p-2 rounded-lg mb-3">
                        <h3 className="text-xs font-semibold mb-1 text-emerald-400">🎁 Free Trial - 3 Months</h3>
                        <p className="text-xs text-slate-400">
                          Start with a 3-month free trial in Shadow Mode. No credit card required.
                        </p>
                      </div>

                      {subscription ? (
                        <div className="bg-emerald-900/20 border border-emerald-800 p-3 rounded-lg text-center">
                          <CheckCircle2 className="w-8 h-8 text-emerald-400 mx-auto mb-2" />
                          <h3 className="text-sm font-semibold text-emerald-400 mb-1">
                            Trial Started Successfully!
                          </h3>
                          <p className="text-xs text-slate-400 mb-2">
                            Redirecting to dashboard...
                          </p>
                          {subscription.days_remaining !== null && (
                            <p className="text-xs text-slate-500">
                              {subscription.days_remaining} days remaining
                            </p>
                          )}
                        </div>
                      ) : (
                        <>
                          <p className="text-xs text-slate-500 text-center mb-3">
                            Based on technical parameters provided. Does not guarantee regulatory approval.
                          </p>
                          <div className="flex items-start gap-2 mb-3">
                            <input
                              type="checkbox"
                              id="wizard-terms-checkbox"
                              checked={agreedToTerms}
                              onChange={(e) => setAgreedToTerms(e.target.checked)}
                              className="mt-1 w-4 h-4 text-emerald-600 bg-slate-700 border-slate-600 rounded focus:ring-emerald-500"
                            />
                            <label htmlFor="wizard-terms-checkbox" className="text-xs text-slate-400">
                              I agree to the{" "}
                              <a
                                href="/terms"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-emerald-400 hover:text-emerald-300 underline"
                              >
                                Terms of Service
                              </a>
                              {" "}and acknowledge that Veridion Nexus does not provide legal advice.
                            </label>
                          </div>
                          <button
                            onClick={handleNext}
                            disabled={loading || !agreedToTerms}
                            className="w-full bg-emerald-600 hover:bg-emerald-700 text-white py-2 px-4 rounded-lg text-sm font-semibold transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                          >
                            {loading ? (
                              <>
                                <Loader2 className="w-4 h-4 animate-spin" />
                                Starting Trial...
                              </>
                            ) : (
                              <>
                                Start Free Trial
                                <ArrowRight className="w-4 h-4" />
                              </>
                            )}
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </>
              ) : (
                <div className="bg-slate-900 border border-slate-800 rounded-lg p-4 text-center">
                  <p className="text-sm text-slate-400">No recommendations available. Please go back and complete the previous steps.</p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Navigation Buttons */}
        {step < 3 && (
          <div className="mt-4 flex justify-between pt-4 border-t border-slate-800">
            <button
              onClick={handleBack}
              disabled={step === 1 || loading}
              className="px-4 py-1.5 text-sm border border-slate-700 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-800 text-slate-300 transition-all flex items-center gap-2"
            >
              <ArrowLeft className="w-3.5 h-3.5" />
              Back
            </button>
            <button
              onClick={handleNext}
              disabled={loading}
              className="px-4 py-1.5 text-sm bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
            >
              {loading ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  Loading...
                </>
              ) : (
                <>
                  Next
                  <ArrowRight className="w-3.5 h-3.5" />
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </WizardLayout>
  );
}
