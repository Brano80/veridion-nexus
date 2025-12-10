'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

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

  const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

  const handleNext = async () => {
    setError(null);
    setLoading(true);

    try {
      if (step === 1) {
        // Validate step 1
        if (!profile.company_name || !profile.industry || !profile.company_size || !profile.country) {
          setError('Please fill in all required fields');
          setLoading(false);
          return;
        }
        setStep(2);
      } else if (step === 2) {
        // Validate step 2
        if (profile.regulatory_requirements?.length === 0 && profile.ai_use_cases?.length === 0) {
          setError('Please select at least one regulatory requirement or AI use case');
          setLoading(false);
          return;
        }
        setStep(3);
      } else if (step === 3) {
        // Get module recommendations
        const response = await fetch(`${API_BASE}/wizard/recommend-modules`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            industry: profile.industry,
            regulatory_requirements: profile.regulatory_requirements || [],
            ai_use_cases: profile.ai_use_cases || [],
          }),
        });

        if (!response.ok) throw new Error('Failed to get recommendations');
        const data = await response.json();
        setRecommendations(data);
        
        // Auto-select required modules
        const required = data.recommended_modules
          .filter((m: RecommendedModule) => m.priority === 'REQUIRED')
          .map((m: RecommendedModule) => m.module_name);
        setSelectedModules(required);
        setStep(4);
      } else if (step === 4) {
        // Calculate pricing
        const response = await fetch(`${API_BASE}/wizard/calculate-price`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            selected_modules: selectedModules,
            num_systems: profile.estimated_ai_systems || 1,
          }),
        });

        if (!response.ok) throw new Error('Failed to calculate price');
        const data = await response.json();
        setPricing(data);
        setStep(5);
      } else if (step === 5) {
        // Create company profile and start trial
        const profileResponse = await fetch(`${API_BASE}/wizard/company-profile`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(profile),
        });

        if (!profileResponse.ok) throw new Error('Failed to create company profile');
        const profileData = await profileResponse.json();
        setCompanyId(profileData.id);

        // Start trial
        const trialResponse = await fetch(`${API_BASE}/wizard/start-trial`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            company_id: profileData.id,
            selected_modules: selectedModules,
            estimated_ai_systems: profile.estimated_ai_systems || 1,
          }),
        });

        if (!trialResponse.ok) throw new Error('Failed to start trial');
        const trialData = await trialResponse.json();
        setSubscription(trialData);
        setStep(6);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  const handleBack = () => {
    if (step > 1) setStep(step - 1);
  };

  const toggleModule = (moduleName: string) => {
    setSelectedModules(prev => 
      prev.includes(moduleName)
        ? prev.filter(m => m !== moduleName)
        : [...prev, moduleName]
    );
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4">
      <div className="max-w-4xl mx-auto">
        <div className="bg-white rounded-lg shadow-xl p-8">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">Veridion Nexus Setup Wizard</h1>
            <p className="text-gray-600">Get started in minutes with automated compliance</p>
          </div>

          {/* Progress Bar */}
          <div className="mb-8">
            <div className="flex justify-between mb-2">
              {[1, 2, 3, 4, 5, 6].map((s) => (
                <div key={s} className={`flex-1 h-2 mx-1 rounded ${step >= s ? 'bg-blue-600' : 'bg-gray-200'}`} />
              ))}
            </div>
            <div className="text-sm text-gray-600 text-center">
              Step {step} of 6
            </div>
          </div>

          {error && (
            <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
              {error}
            </div>
          )}

          {/* Step 1: Company Info */}
          {step === 1 && (
            <div className="space-y-6">
              <h2 className="text-2xl font-semibold">Company Information</h2>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Company Name *</label>
                <input
                  type="text"
                  value={profile.company_name}
                  onChange={(e) => setProfile({ ...profile, company_name: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                  placeholder="Acme Corp"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Industry *</label>
                <select
                  value={profile.industry}
                  onChange={(e) => setProfile({ ...profile, industry: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select industry</option>
                  {INDUSTRIES.map(industry => (
                    <option key={industry.value} value={industry.value}>{industry.label}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Company Size *</label>
                <select
                  value={profile.company_size}
                  onChange={(e) => setProfile({ ...profile, company_size: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select size</option>
                  {COMPANY_SIZES.map(size => (
                    <option key={size.value} value={size.value}>{size.label}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Country *</label>
                <input
                  type="text"
                  value={profile.country}
                  onChange={(e) => setProfile({ ...profile, country: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                  placeholder="Slovakia"
                />
              </div>
            </div>
          )}

          {/* Step 2: Regulatory Requirements & Use Cases */}
          {step === 2 && (
            <div className="space-y-6">
              <h2 className="text-2xl font-semibold">Regulatory Requirements</h2>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Which regulations apply to your business?
                </label>
                <div className="grid grid-cols-2 gap-3">
                  {REGULATORY_REQUIREMENTS.map(req => (
                    <label key={req.value} className="flex items-center p-3 border border-gray-300 rounded-lg cursor-pointer hover:bg-gray-50">
                      <input
                        type="checkbox"
                        checked={profile.regulatory_requirements?.includes(req.value)}
                        onChange={(e) => {
                          const current = profile.regulatory_requirements || [];
                          setProfile({
                            ...profile,
                            regulatory_requirements: e.target.checked
                              ? [...current, req.value]
                              : current.filter(r => r !== req.value)
                          });
                        }}
                        className="mr-2"
                      />
                      <span>{req.label}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  AI Use Cases
                </label>
                <div className="grid grid-cols-2 gap-3">
                  {AI_USE_CASES.map(useCase => (
                    <label key={useCase.value} className="flex items-center p-3 border border-gray-300 rounded-lg cursor-pointer hover:bg-gray-50">
                      <input
                        type="checkbox"
                        checked={profile.ai_use_cases?.includes(useCase.value)}
                        onChange={(e) => {
                          const current = profile.ai_use_cases || [];
                          setProfile({
                            ...profile,
                            ai_use_cases: e.target.checked
                              ? [...current, useCase.value]
                              : current.filter(u => u !== useCase.value)
                          });
                        }}
                        className="mr-2"
                      />
                      <span>{useCase.label}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Estimated Number of AI Systems
                </label>
                <input
                  type="number"
                  min="1"
                  value={profile.estimated_ai_systems}
                  onChange={(e) => setProfile({ ...profile, estimated_ai_systems: parseInt(e.target.value) || 1 })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Deployment Preference</label>
                <select
                  value={profile.deployment_preference}
                  onChange={(e) => setProfile({ ...profile, deployment_preference: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                >
                  {DEPLOYMENT_OPTIONS.map(opt => (
                    <option key={opt.value} value={opt.value}>{opt.label}</option>
                  ))}
                </select>
              </div>
            </div>
          )}

          {/* Step 3: Module Recommendations */}
          {step === 3 && recommendations && (
            <div className="space-y-6">
              <h2 className="text-2xl font-semibold">Recommended Modules</h2>
              <p className="text-gray-600">
                Based on your industry and requirements, we recommend these modules:
              </p>

              <div className="space-y-3">
                {recommendations.recommended_modules.map((module) => (
                  <div
                    key={module.module_name}
                    className={`p-4 border-2 rounded-lg cursor-pointer transition ${
                      selectedModules.includes(module.module_name)
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => toggleModule(module.module_name)}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <input
                            type="checkbox"
                            checked={selectedModules.includes(module.module_name)}
                            onChange={() => toggleModule(module.module_name)}
                            className="mt-1"
                          />
                          <h3 className="font-semibold">{module.display_name}</h3>
                          <span className={`text-xs px-2 py-1 rounded ${
                            module.priority === 'REQUIRED' ? 'bg-red-100 text-red-700' :
                            module.priority === 'RECOMMENDED' ? 'bg-yellow-100 text-yellow-700' :
                            'bg-gray-100 text-gray-700'
                          }`}>
                            {module.priority}
                          </span>
                        </div>
                        {module.description && (
                          <p className="text-sm text-gray-600 ml-6">{module.description}</p>
                        )}
                        <p className="text-xs text-gray-500 ml-6 mt-1">{module.recommendation_reason}</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              <div className="bg-gray-50 p-4 rounded-lg">
                <div className="grid grid-cols-3 gap-4 text-center">
                  <div>
                    <div className="text-2xl font-bold text-red-600">{recommendations.required_count}</div>
                    <div className="text-sm text-gray-600">Required</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-yellow-600">{recommendations.recommended_count}</div>
                    <div className="text-sm text-gray-600">Recommended</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-gray-600">{recommendations.optional_count}</div>
                    <div className="text-sm text-gray-600">Optional</div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Step 4: Pricing Summary */}
          {step === 4 && pricing && (
            <div className="space-y-6">
              <h2 className="text-2xl font-semibold">Pricing Summary</h2>

              <div className="bg-gray-50 p-6 rounded-lg space-y-3">
                <div className="flex justify-between">
                  <span>Base Platform</span>
                  <span className="font-semibold">€{pricing.base_price.toFixed(2)}/month</span>
                </div>
                <div className="flex justify-between">
                  <span>{profile.estimated_ai_systems} AI System(s) × €{pricing.per_system_price.toFixed(2)}</span>
                  <span className="font-semibold">€{(pricing.per_system_price * (profile.estimated_ai_systems || 1)).toFixed(2)}/month</span>
                </div>
                {selectedModules.length > 0 && (
                  <div className="border-t pt-3">
                    <div className="text-sm font-medium mb-2">Selected Modules:</div>
                    {selectedModules.map(moduleName => {
                      const price = pricing.module_prices[moduleName] || 0;
                      return price > 0 ? (
                        <div key={moduleName} className="flex justify-between text-sm">
                          <span>{moduleName.replace('module_', '').replace(/_/g, ' ')}</span>
                          <span>€{price.toFixed(2)}/month</span>
                        </div>
                      ) : null;
                    })}
                  </div>
                )}
                <div className="border-t pt-3 flex justify-between text-lg font-bold">
                  <span>Total (Monthly)</span>
                  <span>€{pricing.total_monthly.toFixed(2)}/month</span>
                </div>
                <div className="border-t pt-3 flex justify-between">
                  <span>Total (Annual)</span>
                  <span className="font-semibold">€{pricing.total_annual.toFixed(2)}/year</span>
                </div>
                <div className="text-sm text-green-600">
                  Save €{pricing.savings_annual.toFixed(2)} with annual billing
                </div>
              </div>

              <div className="bg-blue-50 p-4 rounded-lg">
                <h3 className="font-semibold mb-2">🎁 Free Trial - 3 Months</h3>
                <p className="text-sm text-gray-700">
                  Start with a 3-month free trial in Shadow Mode. No credit card required.
                  You can upgrade to full enforcement at any time.
                </p>
              </div>
            </div>
          )}

          {/* Step 5: Start Trial */}
          {step === 5 && (
            <div className="space-y-6 text-center">
              <h2 className="text-2xl font-semibold">Ready to Start?</h2>
              <p className="text-gray-600">
                Click below to create your account and start your 3-month free trial.
              </p>
            </div>
          )}

          {/* Step 6: Success */}
          {step === 6 && subscription && (
            <div className="space-y-6 text-center">
              <div className="text-6xl mb-4">🎉</div>
              <h2 className="text-2xl font-semibold">Trial Started Successfully!</h2>
              <p className="text-gray-600">
                Your 3-month free trial has been activated. You're now in Shadow Mode.
              </p>

              <div className="bg-green-50 p-6 rounded-lg">
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="font-medium">Trial Status:</span>
                    <span className="font-semibold text-green-600">{subscription.status}</span>
                  </div>
                  {subscription.days_remaining !== null && (
                    <div className="flex justify-between">
                      <span className="font-medium">Days Remaining:</span>
                      <span className="font-semibold">{subscription.days_remaining} days</span>
                    </div>
                  )}
                  {subscription.trial_end_date && (
                    <div className="flex justify-between">
                      <span className="font-medium">Trial Ends:</span>
                      <span className="font-semibold">{new Date(subscription.trial_end_date).toLocaleDateString()}</span>
                    </div>
                  )}
                </div>
              </div>

              <div className="space-y-3">
                <button
                  onClick={() => router.push('/')}
                  className="w-full bg-blue-600 text-white py-3 px-6 rounded-lg font-semibold hover:bg-blue-700 transition"
                >
                  Go to Dashboard
                </button>
                <button
                  onClick={() => router.push('/settings')}
                  className="w-full bg-gray-200 text-gray-700 py-3 px-6 rounded-lg font-semibold hover:bg-gray-300 transition"
                >
                  Configure Settings
                </button>
              </div>
            </div>
          )}

          {/* Navigation Buttons */}
          {step < 6 && (
            <div className="mt-8 flex justify-between">
              <button
                onClick={handleBack}
                disabled={step === 1 || loading}
                className="px-6 py-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
              >
                Back
              </button>
              <button
                onClick={handleNext}
                disabled={loading}
                className="px-6 py-2 bg-blue-600 text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-700"
              >
                {loading ? 'Loading...' : step === 5 ? 'Start Free Trial' : 'Next'}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

